use crayon::resource::workflow;
use bincode;

use shader_compiler::*;
use shader_compiler::backend::ShaderBackend;

use std::path::Path;
use errors::*;
use super::ResourceDatabase;
use super::metadata::ResourceUnderlyingMetadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderMetadata;

impl ShaderMetadata {
    pub fn new() -> Self {
        ShaderMetadata {}
    }
}

impl ResourceUnderlyingMetadata for ShaderMetadata {
    fn validate(&self, _: &[u8]) -> Result<()> {
        Ok(())
    }

    fn build(&self,
             _: &ResourceDatabase,
             _: &Path,
             bytes: &[u8],
             mut out: &mut Vec<u8>)
             -> Result<()> {
        let shader = Shader::load(bytes)?;

        let vs = backend::GLSL110::build(&shader, ShaderPhase::Vertex).unwrap();
        let fs = backend::GLSL110::build(&shader, ShaderPhase::Fragment).unwrap();

        let payload = workflow::ShaderSerializationPayload {
            vs: vs,
            fs: fs,
            layout: *shader.layout(),
            render_state: *shader.render_state(),
        };

        bincode::serialize_into(&mut out, &payload, bincode::Infinite)?;
        Ok(())
    }
}