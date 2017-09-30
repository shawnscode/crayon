/// Raw bytes.
pub type Bytes = Vec<u8>;

impl super::super::Resource for Bytes {
    fn size(&self) -> usize {
        self.len()
    }
}