use std::fs;
use std::path::Path;
use std::io::Read;

use toml;
use bincode;
use crayon::resource::workflow;
use crayon::{graphics, math};
use crayon::math::{Zero, One};

use errors::*;
use workspace::Database;
use super::ResourceUnderlyingMetadata;
use shaderc::Shader;

use utils::toml::*;

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
            let sub_path = load_as_str(&value, &["Shader", "path"])
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
        let mut uniforms = Vec::new();

        if let Some(uniform_table) = load(&value, &["Uniforms"]) {
            for (name, tt) in shader.uniforms() {
                match tt {
                    &graphics::UniformVariableType::Texture => {
                        let uuid = load_as_str(&uniform_table, &[&name])
                            .and_then(|texture_path| {
                                          path.parent()
                                              .and_then(|v| Some(v.join(texture_path)))
                                              .and_then(|v| database.uuid(v))
                                      });

                        textures.push((name.clone(), uuid));
                    }
                    &graphics::UniformVariableType::F32 => {
                        let value = load_as_f32(&uniform_table, &[&name]).unwrap_or(0f32);
                        uniforms.push((name.clone(), graphics::UniformVariable::F32(value)));
                    }
                    &graphics::UniformVariableType::I32 => {
                        let value = load_as_i32(&uniform_table, &[&name]).unwrap_or(0);
                        uniforms.push((name.clone(), graphics::UniformVariable::I32(value)));
                    }
                    &graphics::UniformVariableType::Vector2f => {
                        let value = load_as_vec2(&uniform_table, &[&name])
                            .unwrap_or(math::Vector2::zero());
                        uniforms.push((name.clone(),
                                       graphics::UniformVariable::Vector2f(*value.as_ref())));
                    }
                    &graphics::UniformVariableType::Vector3f => {
                        let value = load_as_vec3(&uniform_table, &[&name])
                            .unwrap_or(math::Vector3::zero());
                        uniforms.push((name.clone(),
                                       graphics::UniformVariable::Vector3f(*value.as_ref())));
                    }
                    &graphics::UniformVariableType::Vector4f => {
                        let value = load_as_vec4(&uniform_table, &[&name])
                            .unwrap_or(math::Vector4::zero());
                        uniforms.push((name.clone(),
                                       graphics::UniformVariable::Vector4f(*value.as_ref())));
                    }
                    &graphics::UniformVariableType::Matrix2f => {
                        let value = load_as_mat2(&uniform_table, &[&name])
                            .unwrap_or(math::Matrix2::one());
                        uniforms.push((name.clone(),
                                       graphics::UniformVariable::Matrix2f(*value.as_ref(),
                                                                           false)));
                    }
                    &graphics::UniformVariableType::Matrix3f => {
                        let value = load_as_mat3(&uniform_table, &[&name])
                            .unwrap_or(math::Matrix3::one());
                        uniforms.push((name.clone(),
                                       graphics::UniformVariable::Matrix3f(*value.as_ref(),
                                                                           false)));
                    }
                    &graphics::UniformVariableType::Matrix4f => {
                        let value = load_as_mat4(&uniform_table, &[&name])
                            .unwrap_or(math::Matrix4::one());
                        uniforms.push((name.clone(),
                                       graphics::UniformVariable::Matrix4f(*value.as_ref(),
                                                                           false)));
                    }
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