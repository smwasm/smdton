use json::JsonValue;

use crate::{SmDtonBuffer, SmDtonPair, SmDtonReader};

macro_rules! def_get_func {
    ($func_name:ident, $dty: ty) => {
        #[allow(dead_code)]
        pub fn $func_name(&self, key: &str) -> Option<$dty> {
            match &self.nread {
                Some(q) => {
                    let op = q.$func_name(1, key);
                    match op {
                        Some(data) => return Some(data),
                        _ => {}
                    }
                }
                _ => {}
            }
            match &self.oread {
                Some(q) => {
                    let op = q.$func_name(1, key);
                    match op {
                        Some(data) => return Some(data),
                        _ => {}
                    }
                }
                _ => {}
            }
            return None
        }
    };
}

#[allow(dead_code)]
pub struct SmDton<'a> {
    oread: Option<SmDtonReader<'a>>,
    nread: Option<SmDtonReader<'a>>,
}

impl<'a> SmDton<'a> {
    pub fn new_from_buffer(smb: &'a SmDtonBuffer) -> Self {
        let buf = smb.get_buffer();
        if buf.len() > 0 {
            SmDton {
                oread: Some(SmDtonReader::new(buf)),
                nread: None,
            }
        } else {
            SmDton {
                oread: None,
                nread: None,
            }
        }
    }

    pub fn new_from_pair(pair: &'a SmDtonPair) -> Self {
        let buf1 = pair.raw.get_buffer();
        let buf2 = pair.update.get_buffer();
        if buf1.len() > 0 {
            if buf2.len() > 0 {
                SmDton {
                    oread: Some(SmDtonReader::new(buf1)),
                    nread: Some(SmDtonReader::new(buf2)),
                }
            } else {
                SmDton {
                    oread: Some(SmDtonReader::new(buf1)),
                    nread: None,
                }
            }
        } else {
            SmDton {
                oread: None,
                nread: None,
            }
        }
    }

    pub fn update(&mut self, vec: &'a [u8]) {
        self.nread = Some(SmDtonReader::new(vec));
    }

    pub fn update_by_dton(&mut self, ndt: &SmDton<'a>) {
        if self.oread.is_some() {
            match &ndt.oread {
                Some(rd) => {
                    self.nread = Some(rd.clone());
                }
                _ => {}
            }
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match &self.nread {
            Some(q) => {
                let op = q.get_string(1, key);
                match op {
                    Some(data) => return Some(data.to_string()),
                    _ => {}
                }
            }
            None => {}
        }
        match &self.oread {
            Some(q) => {
                let op = q.get_string(1, key);
                match op {
                    Some(data) => return Some(data.to_string()),
                    _ => {}
                }
            }
            None => {}
        }
        return None;
    }

    def_get_func!(get_bool, bool);
    def_get_func!(get_u8, u8);
    def_get_func!(get_bin, &[u8]);

    def_get_func!(get_i16, i16);
    def_get_func!(get_u16, u16);

    def_get_func!(get_i32, i32);
    def_get_func!(get_u32, u32);
    def_get_func!(get_f32, f32);

    def_get_func!(get_i64, i64);
    def_get_func!(get_u64, u64);
    def_get_func!(get_f64, f64);

    pub fn stringify(&self) -> Option<String> {
        let mut ret: Option<JsonValue> = None;
        match &self.oread {
            Some(q) => {
                let op = q.to_json(1);
                match op {
                    Some(jsn) => {
                        ret = Some(jsn);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        match &self.nread {
            Some(q) => {
                let op = q.to_json(1);
                match op {
                    Some(jsn) => {
                        if ret.is_some() {
                            let mut r = ret.unwrap();
                            for x in jsn.entries() {
                                let k = x.0.to_string();
                                let v = x.1;

                                r[k] = v.clone();
                            }
                            return Some(r.to_string());
                        } else {
                            return Some(jsn.to_string());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if ret.is_some() {
            return Some(ret.unwrap().pretty(4));
        }
        return None;
    }

    pub fn combine<'b>(&self, ndt: &SmDton<'a>) -> Option<JsonValue> {
        let ops1 = self.stringify();
        let ops2 = ndt.stringify();

        let mut s1 = "".to_string();
        if ops1.is_some() {
            s1 = ops1.unwrap();
        }
        let mut s2 = "".to_string();
        if ops2.is_some() {
            s2 = ops2.unwrap();
        }

        let r1 = json::parse(&s1);
        let r2 = json::parse(&s2);
        match r1 {
            Ok(mut jsn1) => {
                match r2 {
                    Ok(jsn2) => {
                        for x in jsn2.entries() {
                            let k = x.0.to_string();
                            let v = x.1;

                            jsn1[k] = v.clone();
                        }
                    }
                    _ => {}
                }
                return Some(jsn1);
            }
            _ => match r2 {
                Ok(jsn2) => {
                    return Some(jsn2);
                }
                _ => {
                    return None;
                }
            },
        }
    }

    pub fn clone(&self) -> Self {
        match &self.oread {
            Some(q) => match &self.nread {
                Some(r) => SmDton {
                    oread: Some(q.clone()),
                    nread: Some(r.clone()),
                },
                _ => SmDton {
                    oread: Some(q.clone()),
                    nread: None,
                },
            },
            _ => SmDton {
                oread: None,
                nread: None,
            },
        }
    }
}
