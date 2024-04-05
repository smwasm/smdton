use super::sd_node::SmDtonNode;
use base64::{engine::general_purpose, Engine as _};

pub struct ST {}

impl ST {
    // format type
    pub const SMTY_DTR: u8 = 0x01;

    // node type
    pub const SMDT_MAP: u8 = 0x01;
    pub const SMDT_ARR: u8 = 0x02;

    // data type
    pub const SMDT_BOO: u8 = 0x11; // json
    pub const SMDT_UI8: u8 = 0x12;

    pub const SMDT_I16: u8 = 0x13;
    pub const SMDT_U16: u8 = 0x14; // rare

    pub const SMDT_I32: u8 = 0x15;
    pub const SMDT_U32: u8 = 0x16; // rare
    pub const SMDT_F32: u8 = 0x17; // rare

    pub const SMDT_I64: u8 = 0x18; // json
    pub const SMDT_U64: u8 = 0x19; // rare
    pub const SMDT_F64: u8 = 0x1a; // json

    pub const SMDT_STR: u8 = 0x21; // json
    pub const SMDT_BIN: u8 = 0x22;

    pub const SMDT_B64: u8 = 0xB2;
}

macro_rules! smd_new_data {
    ($smdt: expr, $len: expr, $has_len: expr, $u: expr, $v: expr) => {
        SmDtonData {
            smdt: $smdt,
            len: $len,
            has_len: $has_len,
            u8a: $u,
            v8a: $v,
            oid: 0,
        }
    };
}

macro_rules! def_func_new {
    ($func_name:ident, $smdt: expr, $len: expr, $dty: ty) => {
        #[allow(dead_code)]
        pub fn $func_name(data: $dty) -> Self {
            let v = data.to_le_bytes().to_vec();
            SmDtonData {
                smdt: $smdt,
                len: $len,
                has_len: false,
                u8a: None,
                v8a: Some(v),
                oid: 0,
            }
        }
    };
}

pub struct SmDtonData<'a> {
    pub smdt: u8,
    pub len: usize,
    pub has_len: bool,
    pub u8a: Option<&'a [u8]>,
    pub v8a: Option<Vec<u8>>,
    pub oid: usize,
}

impl<'a> SmDtonData<'a> {
    #[inline]
    pub fn new_bool(data: bool) -> Self {
        let mut d = 0;
        if data {
            d = 1;
        }
        let v: Vec<u8> = vec![d];
        smd_new_data!(ST::SMDT_BOO, 1, false, None, Some(v))
    }

    #[inline]
    pub fn new_u8(data: u8) -> Self {
        let v: Vec<u8> = vec![data];
        smd_new_data!(ST::SMDT_UI8, 1, false, None, Some(v))
    }

    def_func_new!(new_i16, ST::SMDT_I16, 2, i16);
    def_func_new!(new_u16, ST::SMDT_U16, 2, u16);

    def_func_new!(new_i32, ST::SMDT_I32, 4, i32);
    def_func_new!(new_u32, ST::SMDT_U32, 4, u32);
    def_func_new!(new_f32, ST::SMDT_F32, 4, f32);

    def_func_new!(new_i64, ST::SMDT_I64, 8, i64);
    def_func_new!(new_u64, ST::SMDT_U64, 8, u64);
    def_func_new!(new_f64, ST::SMDT_F64, 8, f64);

    #[inline]
    pub fn new_string(data: &'a str) -> Self {
        let u8a = data.as_bytes();
        smd_new_data!(ST::SMDT_STR, u8a.len() + 1, true, Some(u8a), None)
    }

    #[inline]
    pub fn new_bin(data: &'a [u8]) -> Self {
        smd_new_data!(ST::SMDT_BIN, data.len(), true, Some(data), None)
    }

    #[inline]
    pub fn new_b64(data: &str) -> Self {
        let piece = &data[5..];
        let bytes = general_purpose::STANDARD
            .decode(piece)
            .expect("Found invalid");
        SmDtonData {
            smdt: ST::SMDT_B64,
            len: bytes.len(),
            has_len: true,
            u8a: None,
            v8a: Some(bytes),
            oid: 0,
        }
    }

    #[inline]
    pub fn new_node(data: &SmDtonNode) -> Self {
        SmDtonData {
            smdt: data.smdt,
            len: 0,
            has_len: true,
            u8a: None,
            v8a: None,
            oid: data.oid,
        }
    }
}
