macro_rules! def_num_copy {
    ($src: expr, $off: expr, $tgt: expr, $len: expr, $dty: ty) => {
        let bytes: [u8; $len] = $src[$off..$off + $len].try_into().unwrap();
        return <$dty>::from_le_bytes(bytes) as usize;
    };
}

#[inline]
pub fn getblkz(bv: usize, num: usize) -> u8 {
    if bv + num < 256 {
        return 1;
    } else if bv + 2 * num < 65536 {
        return 2;
    } else {
        return 4;
    }
}

#[inline]
pub fn get_int(u8a: &[u8], offset: usize, oz: usize) -> usize {
    match oz {
        1 => {
            return u8a[offset] as usize;
        }
        2 => {
            def_num_copy!(u8a, offset, bytes, 2, u16);
        }
        4 => {
            def_num_copy!(u8a, offset, bytes, 4, u32);
        }
        _ => {}
    }
    return 0;
}
