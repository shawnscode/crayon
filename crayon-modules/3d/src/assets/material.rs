use std::collections::HashMap;

use crayon::utils::HashValue;
use crayon::video::assets::prelude::*;

use assets::pipeline::PipelineHandle;
use errors::*;

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
pub struct Material {
    pub(crate) pipeline: PipelineHandle,
    pub(crate) variables: HashMap<HashValue<str>, UniformVariable>,
    pub(crate) shader_params: ShaderParams,
    pub(crate) update: bool,
}

impl Material {
    pub(crate) fn new(
        pipeline: PipelineHandle,
        variables: HashMap<HashValue<str>, UniformVariable>,
        shader_params: ShaderParams,
    ) -> Material {
        Material {
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

    pub fn variable<T1>(&self, field: T1) -> Option<UniformVariable>
    where
        T1: Into<HashValue<str>>,
    {
        self.variables.get(&field.into()).cloned()
    }

    pub fn variable_name<T1>(&self, field: T1) -> Option<&str>
    where
        T1: Into<HashValue<str>>,
    {
        self.shader_params.uniforms.variable_name(field)
    }

    pub fn variable_type<T1>(&self, field: T1) -> Option<UniformVariableType>
    where
        T1: Into<HashValue<str>>,
    {
        self.shader_params.uniforms.variable_type(field)
    }
}
