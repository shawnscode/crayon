use crayon::resource;
use bincode;

use errors::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct BytesMetadata;

impl BytesMetadata {
    pub fn new() -> BytesMetadata {
        BytesMetadata {}
    }

    pub fn validate(&self, _bytes: &[u8]) -> Result<()> {
        Ok(())
    }

    pub fn build(&self, data: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        let mut bytes = Vec::with_capacity(data.len());
        bytes.copy_from_slice(&data);

        let payload = resource::bytes::BytesSerializationPayload(bytes);

        bincode::serialize_into(&mut out, &payload, bincode::Infinite).unwrap();

        Ok(())
    }
}
