use bincode;

use super::super::errors::*;
use super::super::{ResourceLoader, ResourceFrontend, atlas};

pub type AtlasSerializationPayload = atlas::Atlas;

impl ResourceLoader for AtlasSerializationPayload {
    type Item = atlas::Atlas;

    fn load_from_memory(_: &mut ResourceFrontend, bytes: &[u8]) -> Result<Self::Item> {
        Ok(bincode::deserialize(&bytes)?)
    }
}