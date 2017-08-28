use bincode;

use super::super::errors::*;
use super::super::{ResourceLoader, ResourceSystem, atlas};

pub type AtlasSerializationPayload = atlas::Atlas;

impl ResourceLoader for AtlasSerializationPayload {
    type Item = atlas::Atlas;

    fn load_from_memory(_: &mut ResourceSystem, bytes: &[u8]) -> Result<Self::Item> {
        Ok(bincode::deserialize(&bytes)?)
    }
}