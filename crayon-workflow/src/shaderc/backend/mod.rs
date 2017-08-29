pub mod glsl110;

pub use self::glsl110::GLSL110;

use super::errors::*;
use super::{Shader, ShaderPhase};

pub trait ShaderBackend {
    fn build(shader: &Shader, phase: ShaderPhase) -> Result<String>;
}