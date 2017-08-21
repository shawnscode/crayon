use super::super::atlas;

pub type AtlasSerializationPayload = atlas::Atlas;

impl super::ResourceSerialization for atlas::Atlas {
    type Loader = AtlasSerializationPayload;

    fn payload() -> super::ResourcePayload {
        super::ResourcePayload::Atlas
    }
}