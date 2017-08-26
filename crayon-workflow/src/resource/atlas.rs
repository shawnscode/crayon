use std::collections::HashMap;

use uuid;
use bincode;
use serde_json;
use crayon::resource;

use std::path::Path;
use errors::*;
use super::ResourceDatabase;
use super::metadata::ResourceUnderlyingMetadata;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AtlasPlugins {
    TexturePacker,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtlasMetadata {
    pub plugin: AtlasPlugins,
}

impl ResourceUnderlyingMetadata for AtlasMetadata {
    fn validate(&self, bytes: &[u8]) -> Result<()> {
        match self.plugin {
            AtlasPlugins::TexturePacker => self.validate_with_texture_packer(&bytes),
        }
    }

    fn build(&self,
             database: &ResourceDatabase,
             path: &Path,
             bytes: &[u8],
             mut out: &mut Vec<u8>)
             -> Result<()> {
        match self.plugin {
            AtlasPlugins::TexturePacker => {
                self.build_with_texture_packer(&database, &path, bytes, &mut out)
            }
        }
    }
}

impl AtlasMetadata {
    pub fn new() -> Self {
        AtlasMetadata { plugin: AtlasPlugins::TexturePacker }
    }

    fn validate_with_texture_packer(&self, bytes: &[u8]) -> Result<()> {
        let value: serde_json::Value = serde_json::from_reader(bytes)?;
        let root = value.as_object().unwrap();

        if root.get("meta").and_then(|v| v.as_object()).is_none() {
            bail!("Its not a valid texture packer atlas.");
        }

        if root.get("frames").and_then(|v| v.as_array()).is_none() {
            bail!("Its not a valid texture packer atlas.");
        }

        Ok(())
    }

    fn build_with_texture_packer(&self,
                                 database: &ResourceDatabase,
                                 path: &Path,
                                 bytes: &[u8],
                                 mut out: &mut Vec<u8>)
                                 -> Result<()> {
        let value: serde_json::Value = serde_json::from_reader(bytes)?;
        let root = value.as_object().unwrap();

        let mut scale = 1.0f32;
        let mut texture = uuid::Uuid::nil();
        let mut frames = HashMap::new();

        if let Some(meta) = root.get("meta").and_then(|v| v.as_object()) {
            if let Some(s) = meta.get("scale").and_then(|v| v.as_f64()) {
                scale = s as f32;
            }

            if let Some(image) = meta.get("image").and_then(|v| v.as_str()) {
                let texture_path = path.parent().unwrap().join(Path::new(image));
                if let Some(uuid) = database.uuid(texture_path) {
                    texture = uuid;
                }
            }
        }

        if let Some(table) = root.get("frames").and_then(|v| v.as_array()) {
            for v in table {
                let frame_value = v.get("frame").unwrap();
                let pivot_value = v.get("pivot").unwrap();

                let position = (frame_value["x"].as_u64().unwrap() as u16,
                                frame_value["y"].as_u64().unwrap() as u16);
                let size = (frame_value["w"].as_u64().unwrap() as u16,
                            frame_value["h"].as_u64().unwrap() as u16);
                let pivot = (pivot_value["x"].as_f64().unwrap() as f32,
                             pivot_value["y"].as_f64().unwrap() as f32);

                let frame = resource::atlas::AtlasInternalFrame {
                    position: position,
                    size: size,
                    pivot: pivot,
                };

                let filename = v["filename"].as_str().unwrap().to_owned();
                frames.insert(filename, frame);
            }
        }

        let payload = resource::workflow::AtlasSerializationPayload::new(texture, scale, frames);
        bincode::serialize_into(&mut out, &payload, bincode::Infinite)?;

        Ok(())
    }
}