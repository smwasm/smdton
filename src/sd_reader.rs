use base64::{engine::general_purpose, Engine as _};
use json::JsonValue;
use std::collections::HashMap;

use super::sd_data::ST;
use super::su;

macro_rules! smd_check_type {
    ($off: expr, $self: expr, $smdt: expr) => {
        let body_type = $self.u8a[$off];
        if body_type != $smdt {
            return None;
        }
    };
}

macro_rules! smd_check_node_id {
    ($self: expr, $oid: expr, $ret: expr) => {
        if $oid == 0 || $oid > $self.nnum {
            return $ret;
        }
    };
}

macro_rules! smd_handle_node {
    ($self: expr, $oid: expr, $n_off: ident, $p_off: ident, $sub_num: ident) => {
        let $n_off = $self.node_off + ($oid - 1) * (1 + $self.oz);
        let $p_off = $self.get_int(1 + $n_off);
        let $sub_num = $self.get_int($p_off);
    };
}

macro_rules! def_func_get_by_voff {
    ($func_name:ident, $len: expr, $rty: ty, $smdt: expr) => {
        #[inline]
        pub fn $func_name(&self, value_off: usize) -> Option<$rty> {
            let body_type = self.u8a[value_off];
            if body_type != $smdt {
                return None;
            }
            let mut bytes: [u8; $len] = [0; $len];
            bytes.copy_from_slice(&self.u8a[value_off + 1..value_off + 1 + $len]);
            let d = <$rty>::from_le_bytes(bytes) as $rty;
            return Some(d);
        }
    };
}

macro_rules! def_func_get_by_key {
    ($func_name:ident, $act_name: ident, $rty: ty) => {
        #[allow(dead_code)]
        pub fn $func_name(&self, oid: usize, key: &str) -> Option<$rty> {
            let value_off = self.get_field_voff(oid, key);
            if value_off == 0 {
                return None;
            }
            return self.$act_name(value_off);
        }
    };
}

macro_rules! def_func_get_by_id {
    ($func_name:ident, $act_name: ident, $rty: ty) => {
        #[allow(dead_code)]
        pub fn $func_name(&self, oid: usize, index: usize) -> Option<$rty> {
            let value_off = self.get_sub_voff(oid, index);
            if value_off == 0 {
                return None;
            }
            return self.$act_name(value_off);
        }
    };
}

macro_rules! smd_get_key {
    ($self: expr, $p_off: expr, $index: expr, $txt: ident) => {
        let sub_off = $p_off + (1 + 2 * $index) * $self.oz;
        let key_off = $self.get_int(sub_off);
        let kw = $self.get_int(key_off);

        let piece = &$self.u8a[key_off + $self.oz..key_off + $self.oz + kw - 1];
        let $txt = std::str::from_utf8(piece).unwrap().to_string();
    };
}

macro_rules! smd_add_number {
    ($self: expr, $len: expr, $voff: expr, $rty: ty, $obj: expr, $key: expr) => {
        let mut bytes: [u8; $len] = [0; $len];
        bytes.copy_from_slice(&$self.u8a[$voff + 1..$voff + 1 + $len]);
        let data = <$rty>::from_le_bytes(bytes) as $rty;
        $obj[$key] = JsonValue::from(data);
    };
}

macro_rules! smd_push_number {
    ($self: expr, $len: expr, $voff: expr, $rty: ty, $obj: expr) => {
        let mut bytes: [u8; $len] = [0; $len];
        bytes.copy_from_slice(&$self.u8a[$voff + 1..$voff + 1 + $len]);
        let data = <$rty>::from_le_bytes(bytes) as $rty;
        $obj.push(data).unwrap();
    };
}

pub struct SmDtonReader<'a> {
    u8a: &'a [u8],
    oz: usize,
    nnum: usize,

    node_off: usize,
}

impl<'a> SmDtonReader<'a> {
    #[inline]
    pub fn get_int(&self, offset: usize) -> usize {
        return su::get_int(self.u8a, offset, self.oz);
    }

    // for outside special

    #[inline]
    pub fn get_field_voff(&self, oid: usize, key: &str) -> usize {
        smd_check_node_id!(self, oid, 0);
        let kbs = key.as_bytes();
        let kw = kbs.len() + 1;

        smd_handle_node!(self, oid, n_off, p_off, sub_num);
        for i in (0..sub_num).rev() {
            let p_i_off = p_off + self.oz + self.oz * 2 * i;
            let mut key_off = self.get_int(p_i_off);
            let key_len = self.get_int(key_off);
            if kw == key_len {
                key_off += self.oz;
                let kbody = &self.u8a[key_off..key_off + kw - 1];
                if kbs == kbody {
                    let value_off = self.get_int(p_i_off + self.oz);
                    return value_off;
                }
            }
        }
        return 0;
    }

    #[inline]
    pub fn get_sub_voff(&self, oid: usize, index: usize) -> usize {
        smd_check_node_id!(self, oid, 0);
        smd_handle_node!(self, oid, n_off, p_off, sub_num);
        if index >= sub_num {
            return 0;
        }

        let mut off = index * self.oz;
        let node_type = self.u8a[n_off];
        if node_type == ST::SMDT_MAP {
            off = off * 2 + self.oz;
        }
        let voff = self.get_int(p_off + self.oz + off);
        return voff;
    }

    // for outside

    #[allow(dead_code)]
    pub fn new(u8a: &'a [u8]) -> Self {
        let oz = u8a[1] as usize;

        SmDtonReader {
            u8a: u8a,
            oz: oz,
            nnum: su::get_int(u8a, 2, oz),

            node_off: 3 + 3 * oz,
        }
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> Self {
        SmDtonReader {
            u8a: self.u8a,
            oz: self.oz,
            nnum: self.nnum,

            node_off: self.node_off,
        }
    }

    #[allow(dead_code)]
    pub fn node_type(&self, oid: usize) -> u8 {
        smd_check_node_id!(self, oid, 0);
        let n_off = self.node_off + (oid - 1) * (1 + self.oz);
        let node_type = self.u8a[n_off];
        return node_type;
    }

    #[allow(dead_code)]
    pub fn node_sub_num(&self, oid: usize) -> usize {
        smd_check_node_id!(self, oid, 0);
        smd_handle_node!(self, oid, n_off, p_off, sub_num);
        return sub_num;
    }

    #[allow(dead_code)]
    pub fn get_sub_key(&self, oid: usize, index: usize) -> Option<String> {
        smd_check_node_id!(self, oid, None);
        smd_handle_node!(self, oid, n_off, p_off, sub_num);
        if self.u8a[n_off] != ST::SMDT_MAP || index >= sub_num {
            return None;
        }

        smd_get_key!(self, p_off, index, key);
        return Some(key);
    }

    #[allow(dead_code)]
    pub fn get_sub_map(&self, oid: usize) -> HashMap<String, usize> {
        let mut mp: HashMap<String, usize> = HashMap::default();
        smd_check_node_id!(self, oid, mp);
        smd_handle_node!(self, oid, n_off, p_off, sub_num);
        if self.u8a[n_off] != ST::SMDT_MAP {
            return mp;
        }

        for i in 0..sub_num {
            let off = p_off + (1 + 2 * i) * self.oz;
            let k_off = self.get_int(off);
            let v_off = self.get_int(off + self.oz);
            let kw = self.get_int(k_off);

            let piece = &self.u8a[k_off + self.oz..k_off + self.oz + kw - 1];
            let txt = std::str::from_utf8(piece).unwrap();

            mp.insert(txt.to_string(), v_off);
        }

        return mp;
    }

    #[allow(dead_code)]
    pub fn to_json(&self, oid: usize) -> Option<JsonValue> {
        smd_check_node_id!(self, oid, None);
        smd_handle_node!(self, oid, n_off, p_off, sub_num);

        match self.u8a[n_off] {
            ST::SMDT_MAP => {
                let mut obj = JsonValue::new_object();
                for index in 0..sub_num {
                    smd_get_key!(self, p_off, index, key);
                    let voff = self.get_int(p_off + self.oz * 2 * (1 + index));

                    match self.u8a[voff] {
                        ST::SMDT_I16 => {
                            smd_add_number!(self, 2, voff, i16, obj, key);
                        }
                        ST::SMDT_U16 => {
                            smd_add_number!(self, 2, voff, u16, obj, key);
                        }
                        ST::SMDT_I32 => {
                            smd_add_number!(self, 4, voff, i32, obj, key);
                        }
                        ST::SMDT_U32 => {
                            smd_add_number!(self, 4, voff, u32, obj, key);
                        }
                        ST::SMDT_F32 => {
                            smd_add_number!(self, 4, voff, f32, obj, key);
                        }
                        ST::SMDT_I64 => {
                            smd_add_number!(self, 8, voff, i64, obj, key);
                        }
                        ST::SMDT_U64 => {
                            smd_add_number!(self, 8, voff, u64, obj, key);
                        }
                        ST::SMDT_F64 => {
                            smd_add_number!(self, 8, voff, f64, obj, key);
                        }
                        ST::SMDT_BIN => {
                            let bytes = self.get_bin_by_voff(voff).unwrap();
                            let data = general_purpose::STANDARD.encode(&bytes);
                            obj[key] = JsonValue::from("$B64$".to_string() + &data);
                        }
                        ST::SMDT_BOO => {
                            let data = self.u8a[voff + 1] == 1;
                            obj[key] = JsonValue::from(data);
                        }
                        ST::SMDT_UI8 => {
                            obj[key] = JsonValue::from(self.u8a[voff + 1]);
                        }
                        ST::SMDT_STR => {
                            let data = self.get_string_by_voff(voff).unwrap();
                            obj[key] = JsonValue::from(data);
                        }
                        ST::SMDT_MAP | ST::SMDT_ARR => {
                            let next_oid = self.get_int(voff + 1);
                            let data = self.to_json(next_oid).unwrap();
                            obj[key] = data;
                        }
                        _ => {}
                    }
                }
                return Some(obj);
            }
            ST::SMDT_ARR => {
                let mut obj = JsonValue::new_array();

                for index in 0..sub_num {
                    let voff = self.get_int(p_off + (index + 1) * self.oz);

                    match self.u8a[voff] {
                        ST::SMDT_I16 => {
                            smd_push_number!(self, 2, voff, i16, obj);
                        }
                        ST::SMDT_U16 => {
                            smd_push_number!(self, 2, voff, u16, obj);
                        }
                        ST::SMDT_I32 => {
                            smd_push_number!(self, 4, voff, i32, obj);
                        }
                        ST::SMDT_U32 => {
                            smd_push_number!(self, 4, voff, u32, obj);
                        }
                        ST::SMDT_F32 => {
                            smd_push_number!(self, 4, voff, f32, obj);
                        }
                        ST::SMDT_I64 => {
                            smd_push_number!(self, 8, voff, i64, obj);
                        }
                        ST::SMDT_U64 => {
                            smd_push_number!(self, 8, voff, u64, obj);
                        }
                        ST::SMDT_F64 => {
                            smd_push_number!(self, 8, voff, f64, obj);
                        }
                        ST::SMDT_BOO => {
                            let data = self.u8a[voff + 1] == 1;
                            obj.push(data).unwrap();
                        }
                        ST::SMDT_UI8 => {
                            obj.push(self.u8a[voff + 1]).unwrap();
                        }
                        ST::SMDT_STR => {
                            let data = self.get_string_by_voff(voff).unwrap();
                            obj.push(data).unwrap();
                        }
                        ST::SMDT_MAP | ST::SMDT_ARR => {
                            let next_oid = self.get_int(voff + 1);
                            let data = self.to_json(next_oid).unwrap();
                            obj.push(data).unwrap();
                        }
                        _ => {}
                    }
                }
                return Some(obj);
            }
            _ => {}
        }

        return None;
    }

    // get value from value offset

    #[inline]
    pub fn get_type_by_voff(&self, value_off: usize) -> Option<u8> {
        return Some(self.u8a[value_off]);
    }

    #[inline]
    pub fn get_bool_by_voff(&self, value_off: usize) -> Option<bool> {
        smd_check_type!(value_off, self, ST::SMDT_BOO);
        return Some(self.u8a[value_off + 1] == 1);
    }

    #[inline]
    pub fn get_u8_by_voff(&self, value_off: usize) -> Option<u8> {
        smd_check_type!(value_off, self, ST::SMDT_UI8);
        return Some(self.u8a[value_off + 1]);
    }

    #[inline]
    pub fn get_string_by_voff(&self, value_off: usize) -> Option<&str> {
        smd_check_type!(value_off, self, ST::SMDT_STR);
        let tw = self.get_int(value_off + 1);
        let piece = &self.u8a[value_off + 1 + self.oz..value_off + self.oz + tw];
        let rtxt = std::str::from_utf8(piece);
        match rtxt {
            Ok(txt) => return Some(txt),
            _ => {}
        }
        return None;
    }

    #[allow(dead_code)]
    pub fn get_bin_by_voff(&self, value_off: usize) -> Option<&[u8]> {
        smd_check_type!(value_off, self, ST::SMDT_BIN);
        let len = self.get_int(value_off + 1);
        let piece = &self.u8a[value_off + 1 + self.oz..value_off + 1 + self.oz + len];
        return Some(piece);
    }

    #[allow(dead_code)]
    pub fn get_node_id_by_voff(&self, value_off: usize) -> Option<usize> {
        let body_type = self.u8a[value_off];
        if body_type != ST::SMDT_MAP && body_type != ST::SMDT_ARR {
            return None;
        }

        let oid = self.get_int(value_off + 1);
        return Some(oid);
    }

    def_func_get_by_voff!(get_i16_by_voff, 2, i16, ST::SMDT_I16);
    def_func_get_by_voff!(get_u16_by_voff, 2, u16, ST::SMDT_U16);

    def_func_get_by_voff!(get_i32_by_voff, 4, i32, ST::SMDT_I32);
    def_func_get_by_voff!(get_u32_by_voff, 4, u32, ST::SMDT_U32);
    def_func_get_by_voff!(get_f32_by_voff, 4, f32, ST::SMDT_F32);

    def_func_get_by_voff!(get_i64_by_voff, 8, i64, ST::SMDT_I64);
    def_func_get_by_voff!(get_u64_by_voff, 8, u64, ST::SMDT_U64);
    def_func_get_by_voff!(get_f64_by_voff, 8, f64, ST::SMDT_F64);

    // get value from key

    def_func_get_by_key!(get_bool, get_bool_by_voff, bool);
    def_func_get_by_key!(get_u8, get_u8_by_voff, u8);

    def_func_get_by_key!(get_i16, get_i16_by_voff, i16);
    def_func_get_by_key!(get_u16, get_u16_by_voff, u16);

    def_func_get_by_key!(get_i32, get_i32_by_voff, i32);
    def_func_get_by_key!(get_u32, get_u32_by_voff, u32);
    def_func_get_by_key!(get_f32, get_f32_by_voff, f32);

    def_func_get_by_key!(get_i64, get_i64_by_voff, i64);
    def_func_get_by_key!(get_u64, get_u64_by_voff, u64);
    def_func_get_by_key!(get_f64, get_f64_by_voff, f64);

    def_func_get_by_key!(get_string, get_string_by_voff, &str);
    def_func_get_by_key!(get_bin, get_bin_by_voff, &[u8]);
    def_func_get_by_key!(get_node_id, get_node_id_by_voff, usize);

    // get value from index

    def_func_get_by_id!(get_bool_by_id, get_bool_by_voff, bool);
    def_func_get_by_id!(get_u8_by_id, get_u8_by_voff, u8);

    def_func_get_by_id!(get_i16_by_id, get_i16_by_voff, i16);
    def_func_get_by_id!(get_u16_by_id, get_u16_by_voff, u16);

    def_func_get_by_id!(get_i32_by_id, get_i32_by_voff, i32);
    def_func_get_by_id!(get_u32_by_id, get_u32_by_voff, u32);
    def_func_get_by_id!(get_f32_by_id, get_f32_by_voff, f32);

    def_func_get_by_id!(get_i64_by_id, get_i64_by_voff, i64);
    def_func_get_by_id!(get_u64_by_id, get_u64_by_voff, u64);
    def_func_get_by_id!(get_f64_by_id, get_f64_by_voff, f64);

    def_func_get_by_id!(get_string_by_id, get_string_by_voff, &str);
    def_func_get_by_id!(get_bin_by_id, get_bin_by_voff, &[u8]);
    def_func_get_by_key!(get_node_id_by_id, get_node_id_by_voff, usize);
}
