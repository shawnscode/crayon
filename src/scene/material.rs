use std::collections::HashMap;

use graphics::{RenderState, ShaderHandle, ShaderState, UniformVariable, UniformVariableType};
use utils::HashValue;

use scene::errors::*;

impl_handle!(MaterialHandle);

#[derive(Debug, Clone)]
pub struct Material {
    shader: ShaderHandle,
    render_state: RenderState,
    fields: HashMap<HashValue<str>, UniformVariableType>,
    pub(crate) variables: HashMap<HashValue<str>, UniformVariable>,
}

impl Material {
    pub fn new(shader: ShaderHandle, state: ShaderState) -> Self {
        Material {
            shader: shader,
            render_state: state.render_state,
            fields: state.uniform_variables,
            variables: HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn shader(&self) -> ShaderHandle {
        self.shader
    }

    #[inline(always)]
    pub fn render_state(&self) -> RenderState {
        self.render_state
    }

    #[inline(always)]
    pub fn has_uniform_variable<T1>(&self, field: T1) -> bool
    where
        T1: Into<HashValue<str>>,
    {
        self.fields.contains_key(&field.into())
    }

    pub fn set_uniform_variable<T1, T2>(&mut self, field: T1, variable: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        let field = field.into();
        let variable = variable.into();

        if !self.fields.contains_key(&field) {
            bail!(ErrorKind::UniformUndefined);
        }

        if let Some(&tt) = self.fields.get(&field) {
            if tt != variable.variable_type() {
                bail!(ErrorKind::UniformTypeInvalid);
            }
        }

        self.variables.insert(field, variable);
        Ok(())
    }

    #[inline(always)]
    pub fn uniform_variable<T1>(&self, field: T1) -> Option<UniformVariable>
    where
        T1: Into<HashValue<str>>,
    {
        self.variables.get(&field.into()).map(|v| *v)
    }
}
