use std::collections::HashMap;

use crayon::graphics::assets::prelude::*;
use crayon::utils::HashValue;

use errors::*;
use assets::pipeline::PipelineHandle;

impl_handle!(MaterialHandle);

#[derive(Debug, Clone)]
pub struct MaterialSetup {
    pub(crate) pipeline: PipelineHandle,
    pub(crate) variables: HashMap<HashValue<str>, UniformVariable>,
}

impl MaterialSetup {
    pub fn new(pipeline: PipelineHandle) -> MaterialSetup {
        MaterialSetup {
            pipeline: pipeline,
            variables: HashMap::new(),
        }
    }

    pub fn bind<T1, T2>(&mut self, field: T1, variable: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        let field = field.into();
        let variable = variable.into();
        self.variables.insert(field, variable);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MaterialParams {
    pub pipeline: PipelineHandle,
    pub variables: HashMap<HashValue<str>, UniformVariable>,
    pub shader_params: ShaderParams,
    pub update: bool,
}

impl MaterialParams {
    pub fn new(
        pipeline: PipelineHandle,
        variables: HashMap<HashValue<str>, UniformVariable>,
        shader_params: ShaderParams,
    ) -> MaterialParams {
        MaterialParams {
            pipeline: pipeline,
            variables: variables,
            shader_params: shader_params,
            update: true,
        }
    }

    pub fn bind<T1, T2>(&mut self, field: T1, variable: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        let field = field.into();
        let variable = variable.into();

        if let Some(tt) = self.shader_params.uniforms.variable_type(field) {
            if tt != variable.variable_type() {
                return Err(Error::UniformMismatch);
            }
        } else {
            return Err(Error::UniformMismatch);
        }

        self.variables.insert(field, variable);
        self.update = true;
        Ok(())
    }
}
