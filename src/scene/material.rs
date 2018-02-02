use std::collections::HashMap;
use std::sync::Arc;

use graphics::{RenderState, ShaderHandle, UniformVariable};
use utils::HashValue;

use scene::errors::*;
use scene::scene::RenderShader;
use scene::renderer::RenderUniform;

impl_handle!(MaterialHandle);

#[derive(Debug, Clone)]
pub struct Material {
    pub(crate) shader: Arc<RenderShader>,
    pub(crate) variables: HashMap<HashValue<str>, UniformVariable>,
}

impl Material {
    pub fn new(shader: Arc<RenderShader>) -> Self {
        Material {
            shader: shader,
            variables: HashMap::new(),
        }
    }

    #[inline]
    pub fn shader(&self) -> ShaderHandle {
        self.shader.handle
    }

    #[inline]
    pub fn render_state(&self) -> &RenderState {
        self.shader.sso.render_state()
    }

    #[inline]
    pub fn has_uniform_variable<T1>(&self, field: T1) -> bool
    where
        T1: Into<HashValue<str>>,
    {
        self.shader.sso.uniform_variable(field).is_some()
    }

    pub fn set_uniform_variable<T1, T2>(&mut self, field: T1, variable: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        let field = field.into();
        let variable = variable.into();

        if let Some(tt) = self.shader.sso.uniform_variable(field) {
            if tt != variable.variable_type() {
                bail!(ErrorKind::UniformTypeInvalid);
            }
        } else {
            bail!(ErrorKind::UniformUndefined);
        }

        self.variables.insert(field, variable);
        Ok(())
    }

    #[inline]
    pub fn uniform_variable<T1>(&self, field: T1) -> Option<UniformVariable>
    where
        T1: Into<HashValue<str>>,
    {
        self.variables.get(&field.into()).cloned()
    }

    #[inline]
    pub(crate) fn render_uniform_field(&self, uniform: RenderUniform) -> HashValue<str> {
        self.shader
            .render_uniforms
            .get(&uniform)
            .cloned()
            .unwrap_or_else(|| RenderUniform::FIELDS[uniform as usize].into())
    }
}
