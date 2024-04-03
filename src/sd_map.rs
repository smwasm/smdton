use json::JsonValue;

use super::{
    sd_buffer::SmDtonBuffer,
    sd_data::{SmDtonData, ST},
    su,
};

macro_rules! def_map_add {
    ($func_name:ident, $new_name:ident, $dty: ty) => {
        #[allow(dead_code)]
        pub fn $func_name(&mut self, key: &'a str, value: $dty) {
            self._do_add(key, SmDtonData::$new_name(value));
        }
    };
}

pub struct SmDtonMap<'a> {
    // key part
    keys: Vec<&'a str>,
    kdata_size: usize,

    // value part
    values: Vec<SmDtonData<'a>>,
    vdata_size: usize,
    len_blk: usize,
}

impl<'a> SmDtonMap<'a> {
    pub fn build(&mut self) -> SmDtonBuffer {
        let mut smb = SmDtonBuffer::new();
        let knum = self.keys.len();
        let vnum = self.values.len();

        let total = 7 + self.kdata_size + vnum + self.vdata_size;
        let onum = 4 + knum + self.len_blk;

        let p_onum = 1 + knum + vnum;

        // calc oz
        let oz = su::getblkz(total, onum + p_onum) as usize;
        // total size
        let size = total + oz * (onum + p_onum);

        // build head
        smb.build_start(size, oz);
        smb.build_put_int(1);
        smb.build_put_int(knum);
        smb.build_put_int(vnum);
        smb.build_put_u8(0x77);

        // build node head
        let p_off = 4 + 4 * oz;
        smb.build_put_u8(ST::SMDT_MAP);
        smb.build_put_int(p_off);

        // build node pieces
        let kseg_off = 5 + (4 + p_onum) * oz;
        let vseg_off = kseg_off + knum * oz + self.kdata_size + 1;

        let kseg_offs = smb.calc_key_part(knum, kseg_off, &self.keys);
        let vseg_offs = smb.calc_value_part(vnum, vseg_off, &self.values);

        smb.build_put_int(vnum);
        for k in 0..vnum {
            if self.keys.len() > 0 {
                smb.build_put_int(kseg_offs[k]);
            }
            smb.build_put_int(vseg_offs[k]);
        }
        smb.build_put_u8(0x77);

        // build key segment & value segment
        smb.build_kvsegs(knum, vnum, &self.keys, &self.values);

        return smb;
    }

    #[inline]
    fn _do_add(&mut self, key: &'a str, da: SmDtonData<'a>) {
        self.keys.push(key);
        self.kdata_size += key.len() + 1;

        if da.has_len {
            self.len_blk += 1;
        }
        self.vdata_size += da.len;
        self.values.push(da);
    }

    fn _explore_node(&mut self, key: &'a str, jsn: &'a JsonValue) {
        match jsn {
            JsonValue::Boolean(data) => {
                self.add_bool(key, *data);
            }
            JsonValue::Short(s) => {
                self.add_string(key, s);
            }
            JsonValue::String(s) => {
                self.add_string(key, s);
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
                    self.add_i64(key, v);
                } else {
                    let pw = 10f64.powf(exponent as f64);
                    let mut v = mantissa as f64 * pw;
                    if !positive {
                        v = -v;
                    }
                    self.add_f64(key, v);
                }
            }
            _ => {}
        }
    }

    //+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

    pub fn new() -> Self {
        SmDtonMap {
            keys: Vec::with_capacity(16),
            kdata_size: 0,

            values: Vec::with_capacity(16),
            vdata_size: 0,
            len_blk: 0,
        }
    }

    #[allow(dead_code)]
    pub fn add_from_json(&mut self, jsn: &'a JsonValue) {
        match jsn {
            JsonValue::Object(obj) => {
                for (kn, value) in obj.iter() {
                    self._explore_node(kn, value);
                }
            }
            _ => {}
        }
    }

    // add to map
    def_map_add!(add_bool, new_bool, bool);
    def_map_add!(add_u8, new_u8, u8);

    def_map_add!(add_i16, new_i16, i16);
    def_map_add!(add_u16, new_u16, u16);

    def_map_add!(add_i32, new_i32, i32);
    def_map_add!(add_u32, new_u32, u32);
    def_map_add!(add_f32, new_f32, f32);

    def_map_add!(add_i64, new_i64, i64);
    def_map_add!(add_u64, new_u64, u64);
    def_map_add!(add_f64, new_f64, f64);

    def_map_add!(add_string, new_string, &'a str);
    def_map_add!(add_bin, new_bin, &'a [u8]);
}
