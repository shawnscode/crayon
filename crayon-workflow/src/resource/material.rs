use std::fs;
use std::path::Path;
use std::io::Read;

use toml;
use bincode;
use crayon::resource::workflow;
use crayon::graphics;

use errors::*;
use workspace::Database;
use super::ResourceUnderlyingMetadata;
use shaderc::Shader;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialMetadata;

impl MaterialMetadata {
    pub fn new() -> Self {
        MaterialMetadata {}
    }
}

impl ResourceUnderlyingMetadata for MaterialMetadata {
    fn validate(&self, _: &[u8]) -> Result<()> {
        Ok(())
    }

    fn build(&self,
             database: &Database,
             path: &Path,
             bytes: &[u8],
             mut out: &mut Vec<u8>)
             -> Result<()> {

        let value: toml::Value = String::from_utf8_lossy(bytes).parse()?;

        let shader_path = {
            let sub_path = value
                .as_table()
                .and_then(|v| v.get("Shader"))
                .and_then(|v| v.as_table())
                .and_then(|v| v.get("path"))
                .and_then(|v| v.as_str())
                .ok_or(ErrorKind::ShaderNotFound)?;

            path.parent()
                .and_then(|v| Some(v.join(sub_path)))
                .ok_or(ErrorKind::ShaderNotFound)?
        };

        let shader = {
            let mut file = fs::OpenOptions::new().read(true).open(&shader_path)?;

            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Shader::load(&bytes)?
        };

        let mut textures = Vec::new();
        let uniforms = Vec::new();

        if let Some(uniform_table) =
            value
                .as_table()
                .and_then(|v| v.get("Uniforms"))
                .and_then(|v| v.as_table()) {
            for (name, tt) in shader.uniforms() {
                match tt {
                    &graphics::UniformVariableType::Texture => {
                        let uuid = uniform_table
                            .get(name)
                            .and_then(|v| v.as_str())
                            .and_then(|texture_path| {
                                          path.parent()
                                              .and_then(|v| Some(v.join(texture_path)))
                                              .and_then(|v| database.uuid(v))
                                      });

                        textures.push((name.clone(), uuid));
                    }
                    _ => {}
                }
            }
        }

        let payload = workflow::MaterialSerializationPayload {
            shader: database.uuid(shader_path).unwrap(),
            textures: textures,
            uniforms: uniforms,
            priority: 0,
        };

        bincode::serialize_into(&mut out, &payload, bincode::Infinite)?;
        Ok(())
    }
}