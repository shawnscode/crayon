use crayon::resource::workflow;
use bincode;

use shaderc::*;
use shaderc::backend::ShaderBackend;

use std::path::Path;
use errors::*;
use workspace::Database;
use super::ResourceMetadataHandler;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderDesc;

impl ShaderDesc {
    pub fn new() -> Self {
        ShaderDesc {}
    }
}

impl ResourceMetadataHandler for ShaderDesc {
    fn validate(&self, _: &[u8]) -> Result<()> {
        Ok(())
    }

    fn build(&self, _: &Database, _: &Path, bytes: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        let shader = Shader::load(bytes)?;

        let vs = backend::GLSL110::build(&shader, ShaderPhase::Vertex).unwrap();
        let fs = backend::GLSL110::build(&shader, ShaderPhase::Fragment).unwrap();

        let payload = workflow::ShaderSerializationPayload {
            vs: vs,
            fs: fs,
            render_state: *shader.render_state(),
            layout: *shader.layout(),
            uniforms: shader.uniforms().clone(),
        };

        bincode::serialize_into(&mut out, &payload, bincode::Infinite)?;
        Ok(())
    }
}