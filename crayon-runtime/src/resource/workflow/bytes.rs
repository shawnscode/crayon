use super::super::bytes;

pub type BytesSerializationPayload = bytes::Bytes;

impl super::ResourceSerialization for bytes::Bytes {
    type Loader = BytesSerializationPayload;

    fn payload() -> super::ResourcePayload {
        super::ResourcePayload::Bytes
    }
}