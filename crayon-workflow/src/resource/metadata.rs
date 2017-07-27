use std::time::{SystemTime, UNIX_EPOCH};

use uuid;

use super::texture;
use super::Resource;

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceConcreteMetadata {
    Bytes,
    Texture(texture::TextureMetadata),
}

/// The descriptions of a resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMetadata {
    time_created: u64,
    uuid: uuid::Uuid,
    metadata: ResourceConcreteMetadata,
}

impl ResourceMetadata {
    pub fn new(metadata: ResourceConcreteMetadata) -> ResourceMetadata {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ResourceMetadata {
            time_created: timestamp,
            uuid: uuid::Uuid::new_v4(),
            metadata: metadata,
        }
    }

    pub fn new_as(tt: Resource) -> ResourceMetadata {
        let concrete = match tt {
            Resource::Bytes => ResourceConcreteMetadata::Bytes,
            Resource::Texture => ResourceConcreteMetadata::Texture(texture::TextureMetadata::new()),
        };

        ResourceMetadata::new(concrete)
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    pub fn is(&self, tt: Resource) -> bool {
        match self.metadata {
            ResourceConcreteMetadata::Bytes => tt == Resource::Bytes,
            ResourceConcreteMetadata::Texture(_) => tt == Resource::Texture,
        }
    }
}