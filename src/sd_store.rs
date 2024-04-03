use super::sd_data::{SmDtonData, ST};
use std::ptr;

macro_rules! def_mem_copy {
    ($self: expr, $src: expr, $len: expr) => {
        unsafe {
            ptr::copy_nonoverlapping($src.as_ptr(), $self.buf.as_mut_ptr().add($self.off), $len);
        }
        $self.off += $len;
    };
}

pub struct SmDtonStore {
    pub oz: usize,
    pub off: usize,
    pub buf: Vec<u8>,
}

impl SmDtonStore {
    #[inline]
    pub fn new() -> Self {
        SmDtonStore {
            oz: 0,
            off: 0,
            buf: Vec::new(),
        }
    }

    #[inline]
    pub fn build_start(&mut self, size: usize, oz: usize) {
        self.buf = Vec::with_capacity(size);
        unsafe {
            self.buf.set_len(size);
        }

        self.buf[0] = ST::SMTY_DTR;
        self.buf[1] = oz as u8;
        self.off = 2;

        self.oz = oz;
    }

    #[inline(always)]
    pub fn build_put_u8(&mut self, b: u8) {
        self.buf[self.off] = b;
        self.off += 1;
    }

    #[inline(always)]
    pub fn build_put_bin(&mut self, bytes: &[u8], len: usize) {
        def_mem_copy!(self, bytes, len);
    }

    #[inline(always)]
    pub fn build_put_int(&mut self, data: usize) {
        match self.oz {
            1 => {
                self.buf[self.off] = data as u8;
                self.off += 1;
            }
            2 => {
                let bytes = (data as u16).to_le_bytes();
                def_mem_copy!(self, bytes, 2);
            }
            4 => {
                let bytes = (data as u32).to_le_bytes();
                def_mem_copy!(self, bytes, 4);
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub fn calc_key_part<'a>(
        &self,
        knum: usize,
        kseg_off: usize,
        keys: &Vec<&'a str>,
    ) -> Vec<usize> {
        // key part
        let mut kseg_offs: Vec<usize> = Vec::with_capacity(knum);
        unsafe {
            kseg_offs.set_len(knum);
        }
        let mut off = kseg_off;
        for i in 0..knum {
            kseg_offs[i] = off;
            off += self.oz + keys[i].len() + 1;
        }
        return kseg_offs;
    }

    #[inline(always)]
    pub fn calc_value_part<'a>(
        &self,
        vnum: usize,
        vseg_off: usize,
        values: &Vec<SmDtonData<'a>>,
    ) -> Vec<usize> {
        // value part
        let mut vseg_offs: Vec<usize> = Vec::with_capacity(vnum);
        unsafe {
            vseg_offs.set_len(vnum);
        };
        let mut off = vseg_off;
        for i in 0..vnum {
            let vtm = &values[i];
            vseg_offs[i] = off;
            off += 1 + vtm.len;
            if values[i].has_len {
                off += self.oz;
            }
        }
        return vseg_offs;
    }

    #[inline(always)]
    pub fn build_kvsegs<'a>(
        &mut self,
        knum: usize,
        vnum: usize,
        keys: &Vec<&'a str>,
        values: &Vec<SmDtonData<'a>>,
    ) {
        // build key part
        for i in 0..knum {
            let ktm = keys[i].as_bytes();
            self.build_put_int(ktm.len() + 1);
            self.build_put_bin(ktm, ktm.len());
            self.build_put_u8(0);
        }
        self.build_put_u8(0x77);

        // build value part
        for i in 0..vnum {
            let vtm = &values[i];
            self.build_put_u8(vtm.smdt);
            if vtm.has_len {
                let mut vlen = vtm.len;
                if vtm.smdt < 0x10 {
                    vlen = vtm.oid;
                }
                self.build_put_int(vlen);
            }

            match vtm.smdt {
                ST::SMDT_STR => {
                    if vtm.u8a.is_none() {
                        continue;
                    }
                    let t = vtm.u8a.as_ref().unwrap();
                    self.build_put_bin(&t, vtm.len - 1);
                    self.build_put_u8(0);
                }
                ST::SMDT_BIN => {
                    if vtm.u8a.is_none() {
                        continue;
                    }
                    let t = vtm.u8a.as_ref().unwrap();
                    self.build_put_bin(&t, vtm.len);
                }
                ST::SMDT_MAP | ST::SMDT_ARR => {}
                _ => {
                    let t = vtm.v8a.as_ref().unwrap();
                    self.build_put_bin(&t, vtm.len);
                }
            }
        }
        self.build_put_u8(0x77);
    }
}
