use json::JsonValue;
use std::collections::HashMap;

use crate::sd_buffer::SmDtonBuffer;

use super::sd_data::{SmDtonData, ST};
use super::sd_node::SmDtonNode;
use super::su;

macro_rules! def_func_add {
    ($func_name:ident, $new_name:ident, $dty: ty) => {
        #[allow(dead_code)]
        pub fn $func_name(&mut self, oid: usize, key: &'a str, value: $dty) {
            if oid == 0 || oid > self.nnum {
                return;
            }
            self._do_add(oid, key, SmDtonData::$new_name(value));
        }
    };
}

macro_rules! def_func_push {
    ($func_name:ident, $new_name:ident, $dty: ty) => {
        #[allow(dead_code)]
        pub fn $func_name(&mut self, oid: usize, value: $dty) {
            if oid == 0 || oid > self.nnum {
                return;
            }
            self._do_push(oid, SmDtonData::$new_name(value));
        }
    };
}

macro_rules! smd_handle_data {
    ($self: expr, $func_add:ident, $func_push:ident, $upoid: expr, $key: expr, $data: expr) => {
        if $key.len() > 0 {
            $self.$func_add($upoid, $key, $data);
        } else {
            $self.$func_push($upoid, $data);
        }
    };
}

pub struct SmDtonBuilder<'a> {
    nodes: Vec<SmDtonNode>,
    nnum: usize,

    // key part
    keys: Vec<&'a str>,
    map: HashMap<&'a str, usize>,
    kdata_size: usize,

    // value part
    values: Vec<SmDtonData<'a>>,
    vdata_size: usize,
    len_blk: usize,
}

impl<'a> SmDtonBuilder<'a> {
    pub fn build(&mut self) -> SmDtonBuffer {
        let mut smb = SmDtonBuffer::new();
        let nnum = self.nodes.len();
        let knum = self.keys.len();
        let vnum = self.values.len();

        let total = 6 + nnum + self.kdata_size + vnum + self.vdata_size;
        let onum = 3 + nnum + knum + self.len_blk;

        let mut node_onum = 0;
        for i in 0..nnum {
            let node = &self.nodes[i];
            node_onum += 1 + node.keys.len() + node.values.len();
        }

        // calc oz
        let oz = su::getblkz(total, onum + node_onum) as usize;
        // total size
        let size = total + oz * (onum + node_onum);

        // build head
        smb.build_start(size, oz);
        smb.build_put_int(nnum);
        smb.build_put_int(knum);
        smb.build_put_int(vnum);
        smb.build_put_u8(0x77);

        // build node head
        let mut p_off = 3 + nnum + (nnum + 3) * oz;
        for i in 0..nnum {
            let node = &self.nodes[i];
            let smdt = node.smdt;
            smb.build_put_u8(smdt);
            smb.build_put_int(p_off);

            p_off += (1 + node.keys.len() + node.values.len()) * oz;
        }

        // build node pieces
        let kseg_off = 3 + nnum + (3 + nnum + node_onum) * oz + 1;
        let vseg_off = kseg_off + knum * oz + self.kdata_size + 1;

        let kseg_offs = smb.calc_key_part(knum, kseg_off, &self.keys);
        let vseg_offs = smb.calc_value_part(vnum, vseg_off, &self.values);

        for i in 0..nnum {
            let node = &self.nodes[i];
            smb.build_put_int(node.values.len());
            for k in 0..node.values.len() {
                if node.keys.len() > 0 {
                    smb.build_put_int(kseg_offs[node.keys[k]]);
                }
                smb.build_put_int(vseg_offs[node.values[k]]);
            }
        }
        smb.build_put_u8(0x77);

        // build key segment & value segment
        smb.build_kvsegs(knum, vnum, &self.keys, &self.values);

        return smb;
    }

    #[inline]
    fn _do_add(&mut self, oid: usize, key: &'a str, da: SmDtonData<'a>) {
        let kid = self._add_key(key);
        let vid = self._add_value(da);
        let ma = &mut self.nodes[oid - 1];
        ma.keys.push(kid);
        ma.values.push(vid);
    }

    #[inline]
    fn _do_push(&mut self, oid: usize, da: SmDtonData<'a>) {
        let vid = self._add_value(da);
        let ma = &mut self.nodes[oid - 1];
        ma.values.push(vid);
    }

    #[inline]
    fn _add_key(&mut self, key: &'a str) -> usize {
        let op = self.map.get(key);
        match op {
            Some(di) => {
                return *di;
            }
            _ => {
                let ix = self.keys.len();
                self.keys.push(key);
                self.map.insert(key, ix);
                self.kdata_size += key.len() + 1;
                return ix;
            }
        }
    }

    #[inline]
    fn _add_value(&mut self, da: SmDtonData<'a>) -> usize {
        let ix = self.values.len();
        if da.has_len {
            self.len_blk += 1;
        }
        self.vdata_size += da.len;
        self.values.push(da);
        return ix;
    }

    fn _explore_node(&mut self, upoid: usize, key: &'a str, jsn: &'a JsonValue) {
        match jsn {
            JsonValue::Null => {}
            JsonValue::Boolean(data) => {
                smd_handle_data!(self, add_bool, push_bool, upoid, key, *data);
            }
            JsonValue::Short(s) => {
                if s.starts_with("$B64$") {
                    self.add_base64(upoid, key, s);
                } else {
                    smd_handle_data!(self, add_string, push_string, upoid, key, s);
                }
            }
            JsonValue::String(s) => {
                if s.starts_with("$B64$") {
                    self.add_base64(upoid, key, s);
                } else {
                    smd_handle_data!(self, add_string, push_string, upoid, key, s);
                }
            }
            JsonValue::Number(num) => {
                let (positive, mantissa, exponent) = num.as_parts();
                if exponent >= 0 {
                    let pw = 10u64.pow(exponent as u32);
                    let ab = (mantissa * pw) as i64;
                    let mut v = ab;
                    if !positive {
                        v = -v;
                    }
                    smd_handle_data!(self, add_i64, push_i64, upoid, key, v);
                } else {
                    let pw = 10f64.powf(exponent as f64);
                    let mut v = mantissa as f64 * pw;
                    if !positive {
                        v = -v;
                    }
                    smd_handle_data!(self, add_f64, push_f64, upoid, key, v);
                }
            }
            JsonValue::Object(obj) => {
                let oid = self.create_node(ST::SMDT_MAP);
                if upoid > 0 {
                    smd_handle_data!(self, add_node, push_node, upoid, key, oid);
                }
                for (kn, value) in obj.iter() {
                    self._explore_node(oid, kn, value);
                }
            }
            JsonValue::Array(arr) => {
                let oid = self.create_node(ST::SMDT_ARR);
                if upoid > 0 {
                    smd_handle_data!(self, add_node, push_node, upoid, key, oid);
                }
                for value in arr.iter() {
                    self._explore_node(oid, "", value);
                }
            }
        }
    }

    //+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

    pub fn new() -> Self {
        SmDtonBuilder {
            nodes: Vec::with_capacity(8),
            nnum: 0,

            // key part
            keys: Vec::with_capacity(16),
            map: HashMap::default(),
            kdata_size: 0,

            // value part
            values: Vec::with_capacity(16),
            len_blk: 0,
            vdata_size: 0,
        }
    }

    pub fn new_from_json(jsn: &'a JsonValue) -> Self {
        let mut obj = SmDtonBuilder::new();
        obj._explore_node(0, "", jsn);
        return obj;
    }

    pub fn create_node(&mut self, smdt: u8) -> usize {
        let id = self.nodes.len() + 1;
        let dton = SmDtonNode::new(smdt, id);
        self.nodes.push(dton);
        self.nnum = id;
        return id;
    }

    #[allow(dead_code)]
    pub fn add_from_json(&mut self, oid: usize, key: &'a str, jsn: &'a JsonValue) {
        self._explore_node(oid, key, jsn);
    }

    // add to map node

    def_func_add!(add_bool, new_bool, bool);
    def_func_add!(add_u8, new_u8, u8);

    def_func_add!(add_i16, new_i16, i16);
    def_func_add!(add_u16, new_u16, u16);

    def_func_add!(add_i32, new_i32, i32);
    def_func_add!(add_u32, new_u32, u32);
    def_func_add!(add_f32, new_f32, f32);

    def_func_add!(add_i64, new_i64, i64);
    def_func_add!(add_u64, new_u64, u64);
    def_func_add!(add_f64, new_f64, f64);

    def_func_add!(add_string, new_string, &'a str);
    def_func_add!(add_bin, new_bin, &'a [u8]);
    def_func_add!(add_base64, new_b64, &'a str);

    #[allow(dead_code)]
    pub fn add_node(&mut self, oid: usize, key: &'a str, new_oid: usize) {
        let ma = &self.nodes[new_oid - 1];
        self._do_add(oid, key, SmDtonData::new_node(ma));
    }

    // push to array node

    def_func_push!(push_bool, new_bool, bool);
    def_func_push!(push_u8, new_u8, u8);

    def_func_push!(push_i16, new_i16, i16);
    def_func_push!(push_u16, new_u16, u16);

    def_func_push!(push_i32, new_i32, i32);
    def_func_push!(push_u32, new_u32, u32);
    def_func_push!(push_f32, new_f32, f32);

    def_func_push!(push_i64, new_i64, i64);
    def_func_push!(push_u64, new_u64, u64);
    def_func_push!(push_f64, new_f64, f64);

    def_func_push!(push_string, new_string, &'a str);
    def_func_push!(push_bin, new_bin, &'a [u8]);

    #[allow(dead_code)]
    pub fn push_node(&mut self, oid: usize, new_oid: usize) {
        let ma = &self.nodes[new_oid as usize];
        self._do_push(oid, SmDtonData::new_node(ma));
    }
}
