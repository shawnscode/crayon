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
pub(crate) struct Material {
    pub pipeline: PipelineHandle,
    pub variables: HashMap<HashValue<str>, UniformVariable>,
    pub shader_params: ShaderParams,
    pub update: bool,
}

impl Material {
    pub fn new(
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
}

pub trait MatReader {
    /// Gets the uniform variable of field.
    fn variable<T1>(&self, field: T1) -> Option<UniformVariable>
    where
        T1: Into<HashValue<str>>;

    /// Gets the str name of field.
    fn variable_name<T1>(&self, field: T1) -> Option<&str>
    where
        T1: Into<HashValue<str>>;

    /// Gets the type of field.
    fn variable_type<T1>(&self, field: T1) -> Option<UniformVariableType>
    where
        T1: Into<HashValue<str>>;
}

pub trait MatWriter {
    /// Binds the field with variable.
    fn bind<T1, T2>(&mut self, field: T1, variable: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>;
}

pub struct MatAccessor<'a> {
    mat: &'a Material,
}

impl<'a> MatAccessor<'a> {
    pub(crate) fn new(mat: &'a Material) -> Self {
        MatAccessor { mat: mat }
    }
}

impl<'a> MatReader for MatAccessor<'a> {
    fn variable<T1>(&self, field: T1) -> Option<UniformVariable>
    where
        T1: Into<HashValue<str>>,
    {
        self.mat.variables.get(&field.into()).cloned()
    }

    fn variable_name<T1>(&self, field: T1) -> Option<&str>
    where
        T1: Into<HashValue<str>>,
    {
        self.mat.shader_params.uniforms.variable_name(field)
    }

    fn variable_type<T1>(&self, field: T1) -> Option<UniformVariableType>
    where
        T1: Into<HashValue<str>>,
    {
        self.mat.shader_params.uniforms.variable_type(field)
    }
}

pub struct MatAccessorMut<'a> {
    mat: &'a mut Material,
}

impl<'a> MatAccessorMut<'a> {
    pub(crate) fn new(mat: &'a mut Material) -> Self {
        MatAccessorMut { mat: mat }
    }
}

impl<'a> MatReader for MatAccessorMut<'a> {
    fn variable<T1>(&self, field: T1) -> Option<UniformVariable>
    where
        T1: Into<HashValue<str>>,
    {
        self.mat.variables.get(&field.into()).cloned()
    }

    fn variable_name<T1>(&self, field: T1) -> Option<&str>
    where
        T1: Into<HashValue<str>>,
    {
        self.mat.shader_params.uniforms.variable_name(field)
    }

    fn variable_type<T1>(&self, field: T1) -> Option<UniformVariableType>
    where
        T1: Into<HashValue<str>>,
    {
        self.mat.shader_params.uniforms.variable_type(field)
    }
}

impl<'a> MatWriter for MatAccessorMut<'a> {
    fn bind<T1, T2>(&mut self, field: T1, variable: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        self.mat.bind(field, variable)
    }
}
