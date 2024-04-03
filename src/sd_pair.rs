use crate::SmDtonBuffer;

#[allow(dead_code)]
pub struct SmDtonPair {
    pub raw: SmDtonBuffer,
    pub update: SmDtonBuffer,
}

impl SmDtonPair {
    pub fn new(raw: SmDtonBuffer, update: SmDtonBuffer) -> Self {
        SmDtonPair {
            raw: raw,
            update: update,
        }
    }
}
