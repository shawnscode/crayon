//! Simple wrapper of bytes.

use resource;

/// Just raw bytes.
pub type Bytes = Vec<u8>;

impl resource::Resource for Bytes {
    fn size(&self) -> usize {
        self.len()
    }
}

impl resource::ResourceParser for Bytes {
    type Item = Bytes;

    fn parse(bytes: &[u8]) -> resource::errors::Result<Self::Item> {
        Ok(bytes.to_owned())
    }
}