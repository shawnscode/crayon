use crayon::resource::workflow;
use bincode;

use errors::*;
use shader_compiler::*;
use shader_compiler::backend::ShaderBackend;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderMetadata;

impl ShaderMetadata {
    pub fn new() -> ShaderMetadata {
        ShaderMetadata {}
    }

    pub fn validate(&self, _: &[u8]) -> Result<()> {
        Ok(())
    }

    pub fn build(&self, data: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        let shader = Shader::load(data)?;

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