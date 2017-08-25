use bincode;
use graphics;

use super::super::errors::*;
use super::super::shader;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderSerializationPayload {
    pub vs: String,
    pub fs: String,
    pub layout: graphics::AttributeLayout,
    pub render_state: graphics::RenderState,
}

impl super::super::ResourceLoader for ShaderSerializationPayload {
    type Item = shader::Shader;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        let data: ShaderSerializationPayload = bincode::deserialize(&bytes)?;
        Ok(shader::Shader::new(data.vs, data.fs, &data.render_state, &data.layout))
    }
}

impl super::ResourceSerialization for shader::Shader {
    type Loader = ShaderSerializationPayload;

    fn payload() -> super::ResourcePayload {
        super::ResourcePayload::Shader
    }
}