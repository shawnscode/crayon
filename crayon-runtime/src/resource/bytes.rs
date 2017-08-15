use bincode;

use super::errors::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct BytesSerializationPayload(pub Vec<u8>);

impl super::ResourceLoader for BytesSerializationPayload {
    type Item = Bytes;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        let data: BytesSerializationPayload = bincode::deserialize(&bytes)?;
        let mut n = Vec::with_capacity(data.0.len());
        n.copy_from_slice(&data.0);
        Ok(n)
    }
}

pub type Bytes = Vec<u8>;

impl super::Resource for Bytes {
    fn size(&self) -> usize {
        self.len()
    }
}

impl super::ResourceLoader for Bytes {
    type Item = Bytes;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        BytesSerializationPayload::load_from_memory(&bytes)
    }
}
