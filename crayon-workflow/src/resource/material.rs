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
use super::ResourceMetadataHandler;
use shaderc::Shader;

use utils::toml::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialDesc;

impl MaterialDesc {
    pub fn new() -> Self {
        MaterialDesc {}
    }
}

impl ResourceMetadataHandler for MaterialDesc {
    fn validate(&self, _: &[u8]) -> Result<()> {
        Ok(())
    }

    fn build(&self,
             database: &Database,
             path: &Path,
             bytes: &[u8],
             mut out: &mut Vec<u8>)
             -> Result<()> {

        use self::graphics::UniformVariableType as UVT;
        use self::graphics::UniformVariable as UV;

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
                    &UVT::Texture => {
                        let uuid = load_as_str(&uniform_table, &[&name])
                            .and_then(|texture_path| {
                                          path.parent()
                                              .and_then(|v| Some(v.join(texture_path)))
                                              .and_then(|v| database.uuid(v))
                                      });
                        textures.push((name.clone(), uuid));
                    }
                    &UVT::F32 => {
                        let value = load_as_f32(&uniform_table, &[&name]).unwrap_or(0f32);
                        uniforms.push((name.clone(), graphics::UniformVariable::F32(value)));
                    }
                    &UVT::I32 => {
                        let value = load_as_i32(&uniform_table, &[&name]).unwrap_or(0);
                        uniforms.push((name.clone(), graphics::UniformVariable::I32(value)));
                    }
                    &UVT::Vector2f => {
                        let value = load_as_vec2(&uniform_table, &[&name])
                            .unwrap_or(math::Vector2::zero());
                        uniforms.push((name.clone(), UV::Vector2f(*value.as_ref())));
                    }
                    &UVT::Vector3f => {
                        let value = load_as_vec3(&uniform_table, &[&name])
                            .unwrap_or(math::Vector3::zero());
                        uniforms.push((name.clone(), UV::Vector3f(*value.as_ref())));
                    }
                    &UVT::Vector4f => {
                        let value = load_as_vec4(&uniform_table, &[&name])
                            .unwrap_or(math::Vector4::zero());
                        uniforms.push((name.clone(), UV::Vector4f(*value.as_ref())));
                    }
                    &UVT::Matrix2f => {
                        let value = load_as_mat2(&uniform_table, &[&name])
                            .unwrap_or(math::Matrix2::one());
                        uniforms.push((name.clone(), UV::Matrix2f(*value.as_ref(), false)));
                    }
                    &UVT::Matrix3f => {
                        let value = load_as_mat3(&uniform_table, &[&name])
                            .unwrap_or(math::Matrix3::one());
                        uniforms.push((name.clone(), UV::Matrix3f(*value.as_ref(), false)));
                    }
                    &UVT::Matrix4f => {
                        let value = load_as_mat4(&uniform_table, &[&name])
                            .unwrap_or(math::Matrix4::one());
                        uniforms.push((name.clone(), UV::Matrix4f(*value.as_ref(), false)));
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