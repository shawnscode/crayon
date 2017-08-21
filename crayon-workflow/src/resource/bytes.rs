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
        out.resize(data.len(), 0);
        out.copy_from_slice(&data);
        Ok(())
    }
}
