use bincode;
use graphics;
use uuid;

use super::super::errors::*;
use super::super::{ResourceLoader, ResourceFrontend, material, shader, texture};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialSerializationPayload {
    pub shader: uuid::Uuid,
    pub textures: Vec<(String, Option<uuid::Uuid>)>,
    pub uniforms: Vec<(String, graphics::UniformVariable)>,
    pub priority: i32,
}

impl ResourceLoader for MaterialSerializationPayload {
    type Item = material::Material;

    fn load_from_memory(sys: &mut ResourceFrontend, bytes: &[u8]) -> Result<Self::Item> {
        let data: MaterialSerializationPayload = bincode::deserialize(&bytes)?;
        let shader = sys.load_with_uuid::<shader::Shader>(data.shader)
            .chain_err(|| ErrorKind::ShaderNotFound)?;

        let mut mat = material::Material::new(shader);
        for (name, v) in data.textures {
            let texture = if let Some(uuid) = v {
                sys.load_with_uuid::<texture::Texture>(uuid).ok()
            } else {
                None
            };

            mat.set_texture(&name, texture)?;
        }

        for (name, v) in data.uniforms {
            mat.set_uniform_variable(&name, v)?;
        }

        Ok(mat)
    }
}