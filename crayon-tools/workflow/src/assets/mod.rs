pub mod manifest_compiler;
pub mod texture_compiler;
pub use self::texture_compiler::TextureCompiler;

use std::io::{Read, Write};

use errors::*;

pub trait ResourceCompiler {
    fn compile(&self, i: &mut dyn Read, o: &mut dyn Write) -> Result<()>;
}
