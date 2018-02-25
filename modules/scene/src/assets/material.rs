use std::collections::HashMap;

use crayon::graphics::assets::prelude::*;
use crayon::utils::HashValue;

use errors::*;
use assets::pipeline::{PipelineHandle, PipelineParams};

impl_handle!(MaterialHandle);

#[derive(Debug, Clone)]
pub(crate) struct Material {
    pub pipeline: PipelineHandle,
    pub variables: HashMap<HashValue<str>, UniformVariable>,
    pub update: bool,
}

impl Material {
    pub fn new(pipeline: PipelineHandle) -> Material {
        Material {
            pipeline: pipeline,
            variables: HashMap::new(),
            update: false,
        }
    }

    pub fn set_uniform_variable<T1, T2>(
        &mut self,
        pipeline: &PipelineParams,
        field: T1,
        variable: T2,
    ) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        let field = field.into();
        let variable = variable.into();

        if let Some(tt) = pipeline.shader_params.uniforms.variable_type(field) {
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
