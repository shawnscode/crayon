use std::collections::HashMap;

use bincode;
use graphics;

use super::super::errors::*;
use super::super::{ResourceLoader, ResourceSystem, shader};

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderSerializationPayload {
    pub vs: String,
    pub fs: String,
    pub layout: graphics::AttributeLayout,
    pub render_state: graphics::RenderState,
    pub uniforms: HashMap<String, graphics::UniformVariableType>,
}

impl ResourceLoader for ShaderSerializationPayload {
    type Item = shader::Shader;

    fn load_from_memory(_: &mut ResourceSystem, bytes: &[u8]) -> Result<Self::Item> {
        let data: ShaderSerializationPayload = bincode::deserialize(&bytes)?;
        Ok(shader::Shader::new(data.vs,
                               data.fs,
                               data.render_state,
                               data.layout,
                               data.uniforms))
    }
}