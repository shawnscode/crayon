use std::path::Path;
use errors::*;
use workspace::Database;
use super::ResourceMetadataHandler;

#[derive(Debug, Serialize, Deserialize)]
pub struct BytesDesc;

impl BytesDesc {
    pub fn new() -> Self {
        BytesDesc {}
    }
}

impl ResourceMetadataHandler for BytesDesc {
    fn validate(&self, _: &[u8]) -> Result<()> {
        Ok(())
    }

    fn build(&self, _: &Database, _: &Path, bytes: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        out.resize(bytes.len(), 0);
        out.copy_from_slice(&bytes);
        Ok(())
    }
}