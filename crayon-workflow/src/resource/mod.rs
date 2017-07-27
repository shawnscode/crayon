pub mod texture;
pub use self::texture::TextureMetadata;

use std::path::Path;
use std::fs;
use std::io::{Write, Read};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_yaml;
use uuid;

use errors::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Resource {
    Binary,
    Texture,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceMetadata {
    Binary,
    Texture(texture::TextureMetadata),
    Atlas,
    Prefab,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    time_created: u64,
    uuid: uuid::Uuid,
    metadata: ResourceMetadata,
}

impl Metadata {
    pub fn new(metadata: ResourceMetadata) -> Metadata {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Metadata {
            time_created: timestamp,
            uuid: uuid::Uuid::new_v4(),
            metadata: metadata,
        }
    }

    pub fn deserialize<P>(path: P) -> Result<Metadata>
        where P: AsRef<Path>
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .open(path.as_ref())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(serde_yaml::from_str(&content)?)
    }

    pub fn serialize<P>(&self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path.as_ref())?;
        let serialization = serde_yaml::to_string(&self).unwrap();

        file.write(serialization.as_ref())?;
        file.flush()?;
        Ok(())
    }
}