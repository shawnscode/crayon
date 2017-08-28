use super::super::errors::*;
use super::super::{ResourceLoader, ResourceSystem, bytes};

pub type BytesSerializationPayload = bytes::Bytes;

impl ResourceLoader for BytesSerializationPayload {
    type Item = bytes::Bytes;

    fn load_from_memory(_: &mut ResourceSystem, bytes: &[u8]) -> Result<Self::Item> {
        let mut n = Vec::with_capacity(bytes.len());
        n.copy_from_slice(&bytes);
        Ok(n)
    }
}