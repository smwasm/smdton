
pub struct SmDtonNode {
    pub smdt: u8,
    pub oid: usize,
    pub keys: Vec<usize>,
    pub values: Vec<usize>,
}

impl<'a> SmDtonNode {
    #[inline]
    pub fn new(smdt: u8, oid: usize) -> Self {
        SmDtonNode {
            smdt: smdt,
            oid: oid,
            keys: Vec::with_capacity(4),
            values: Vec::with_capacity(4),
        }
    }
}
