use std::slice::Iter;

use graphics::MAX_UNIFORM_VARIABLES;
use graphics::{ShaderHandle, UniformVariable};

use utils::HashValue;

/// `Material` is used to set custom shader properties, the uniform variables, safely.
#[derive(Debug, Copy, Clone)]
pub struct Material {
    shader: ShaderHandle,
    uniforms: [(HashValue<str>, UniformVariable); MAX_UNIFORM_VARIABLES],
    uniforms_len: usize,
}

impl Material {
    /// Create a new and empty `Material`.
    pub fn new(shader: ShaderHandle) -> Self {
        Material {
            shader: shader,
            uniforms: [(HashValue::zero(), UniformVariable::I32(0)); MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
        }
    }

    /// Get the underlaying shader handle.
    #[inline(always)]
    pub fn shader(&self) -> ShaderHandle {
        self.shader
    }

    /// Returns an iterator over the slice of `UniformVariable`s.
    pub fn iter<'a>(&'a self) -> Iter<'a, (HashValue<str>, UniformVariable)> {
        (self.uniforms[0..self.uniforms_len]).iter()
    }

    /// Bind the named field with `UniformVariable`.
    pub fn set_uniform_variable<T>(&mut self, field: &str, variable: T)
        where T: Into<UniformVariable>
    {
        assert!(self.uniforms_len < MAX_UNIFORM_VARIABLES);

        let field = field.into();
        let variable = variable.into();

        for i in 0..self.uniforms_len {
            if self.uniforms[i].0 == field {
                self.uniforms[i] = (field, variable);
                return;
            }
        }

        self.uniforms[self.uniforms_len] = (field, variable);
        self.uniforms_len += 1;
    }
}