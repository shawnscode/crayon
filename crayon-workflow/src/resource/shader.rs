use crayon::resource::workflow;
use bincode;

use errors::*;
use shader_compiler;

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
        let shader = shader_compiler::Shader::load(data)?;

        let mut vs = String::new();
        shader_compiler::backend::glsl110::write(&mut vs, &shader.vs()).unwrap();

        let mut fs = String::new();
        shader_compiler::backend::glsl110::write(&mut fs, &shader.fs()).unwrap();

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