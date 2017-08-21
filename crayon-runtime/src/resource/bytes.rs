use super::errors::*;

/// Raw bytes.
pub type Bytes = Vec<u8>;

impl super::Resource for Bytes {
    fn size(&self) -> usize {
        self.len()
    }
}

impl super::ResourceLoader for Bytes {
    type Item = Bytes;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        let mut n = Vec::with_capacity(bytes.len());
        n.copy_from_slice(&bytes);
        Ok(n)
    }
}