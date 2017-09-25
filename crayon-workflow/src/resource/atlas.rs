use std::collections::HashMap;

use uuid;
use bincode;
use serde_json;
use crayon::resource;

use std::path::Path;
use errors::*;
use workspace::Database;
use super::ResourceUnderlyingMetadata;

use utils::json::*;

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
             database: &Database,
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
                                 database: &Database,
                                 path: &Path,
                                 bytes: &[u8],
                                 mut out: &mut Vec<u8>)
                                 -> Result<()> {
        let value: serde_json::Value = serde_json::from_reader(bytes)?;
        let root = value.as_object().unwrap();

        let scale = load_as_f32(&value, &["meta", "scale"]).unwrap_or(1f32);
        let texture = load_as_str(&value, &["meta", "image"])
            .and_then(|image| {
                          path.parent()
                              .and_then(|v| Some(v.join(Path::new(image))))
                              .and_then(|v| database.uuid(v))
                      })
            .unwrap_or(uuid::Uuid::nil());

        let mut frames = HashMap::new();

        if let Some(table) = root.get("frames").and_then(|v| v.as_array()) {
            for v in table {
                let position = (load_as_u16(&v, &["frame", "x"]).unwrap_or(0),
                                load_as_u16(&v, &["frame", "y"]).unwrap_or(0));
                let size = (load_as_u16(&v, &["frame", "w"]).unwrap_or(0),
                            load_as_u16(&v, &["frame", "h"]).unwrap_or(0));
                let pivot = (load_as_f32(&v, &["pivot", "x"]).unwrap_or(0f32),
                             load_as_f32(&v, &["pivot", "y"]).unwrap_or(0f32));

                let frame = resource::atlas::AtlasInternalFrame {
                    position: position,
                    size: size,
                    pivot: pivot,
                };

                let filename = load_as_str(&v, &["filename"])
                    .and_then(|v| Some(v.to_owned()))
                    .unwrap();
                frames.insert(filename, frame);
            }
        }

        let payload = resource::workflow::AtlasSerializationPayload::new(texture, scale, frames);
        bincode::serialize_into(&mut out, &payload, bincode::Infinite)?;

        Ok(())
    }
}