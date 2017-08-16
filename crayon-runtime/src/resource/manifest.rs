use std::path::PathBuf;
use std::collections::HashMap;

use uuid;

/// Payload of serialization data.
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum ResourcePayload {
    Bytes,
    Texture,
}

/// A manifest item.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceManifestItem {
    /// The checksum of payload.
    pub checksum: u64,
    /// The original relative path when building resource.
    pub path: PathBuf,
    /// All the dependent resources.
    pub dependencies: Vec<uuid::Uuid>,
    /// Uniqued identifier, this will be used as file name also when building resource.
    pub uuid: uuid::Uuid,
    /// The payload type of this resource.
    pub payload: ResourcePayload,
}

/// Manifest for all the resources in build.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceManifest {
    /// The version of workflow that generated this manifest and corresponding
    /// resources
    pub version: String,
    /// The resource archive path.
    pub path: PathBuf,
    /// All the resources.
    pub items: HashMap<uuid::Uuid, ResourceManifestItem>,
}