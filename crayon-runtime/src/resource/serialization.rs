use std::path::PathBuf;
use std::collections::HashMap;

use uuid;

use super::texture::TextureSerializationData;

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceSerializationData {
    Texture(TextureSerializationData),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceSerializationFile {
    version: String,
    uuid: uuid::Uuid,
    data: ResourceSerializationData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceManifestItem {
    pub checksum: u64,
    pub path: PathBuf,
    pub dependencies: Vec<uuid::Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceManifest {
    pub version: String,
    pub path: PathBuf,
    pub items: HashMap<uuid::Uuid, ResourceManifestItem>,
}