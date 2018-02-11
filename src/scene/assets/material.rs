use std::collections::HashMap;

use graphics::UniformVariable;
use utils::HashValue;

use scene::errors::*;
use scene::assets::pipeline::{PipelineHandle, PipelineObject};

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
        pipeline: &PipelineObject,
        field: T1,
        variable: T2,
    ) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        let field = field.into();
        let variable = variable.into();

        if let Some(tt) = pipeline.sso.uniform_variable(field) {
            if tt != variable.variable_type() {
                bail!(ErrorKind::UniformTypeInvalid);
            }
        } else {
            bail!(ErrorKind::UniformUndefined);
        }

        self.variables.insert(field, variable);
        self.update = true;
        Ok(())
    }
}
