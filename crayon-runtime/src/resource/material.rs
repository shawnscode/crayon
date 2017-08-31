use graphics::{UniformVariable, UniformVariableType};

use super::errors::*;
use super::{TexturePtr, ShaderPtr};

/// `Material` exposes all properties from a shader and allowing you to acess them.
#[derive(Debug, Clone)]
pub struct Material {
    shader: ShaderPtr,
    textures: Vec<(String, Option<TexturePtr>)>,
    uniforms: Vec<(String, UniformVariable)>,
    order: i32,
}

impl Material {
    pub fn new(shader: ShaderPtr) -> Material {
        Material {
            shader: shader,
            textures: Vec::new(),
            uniforms: Vec::new(),
            order: 0,
        }
    }

    /// Render queue of this material, renderer will draw objects in order.
    pub fn render_order(&self) -> i32 {
        self.order
    }

    /// Get the uniforms of this material.
    pub fn uniforms(&self) -> &[(String, UniformVariable)] {
        &self.uniforms
    }

    /// Get the uniform variable with given name.
    pub fn uniform(&self, name: &str) -> Option<UniformVariable> {
        for pair in &self.uniforms {
            if pair.0 == name {
                return Some(pair.1);
            }
        }

        None
    }

    /// Get the textures of this material.
    pub fn textures(&self) -> &[(String, Option<TexturePtr>)] {
        &self.textures
    }

    /// Get the texture with given name.
    pub fn texture(&self, name: &str) -> Option<TexturePtr> {
        for pair in &self.textures {
            if pair.0 == name {
                return pair.1.clone();
            }
        }

        None
    }

    /// Set the texture with given name.
    pub fn set_texture(&mut self, name: &str, texture: Option<TexturePtr>) -> Result<()> {
        let tt = self.shader
            .read()
            .unwrap()
            .uniform_variable(name)
            .ok_or(ErrorKind::UniformVariableNotFound)?;

        if tt != UniformVariableType::Texture {
            bail!(ErrorKind::UniformDeclarationMismatch);
        }

        for pair in &mut self.textures {
            if pair.0 == name {
                pair.1 = texture;
                return Ok(());
            }
        }

        self.textures.push((name.to_owned(), texture));
        Ok(())
    }

    /// Set uniform with variable.
    pub fn set_uniform_variable(&mut self, name: &str, variable: UniformVariable) -> Result<()> {
        let tt = self.shader
            .read()
            .unwrap()
            .uniform_variable(name)
            .ok_or(ErrorKind::UniformVariableNotFound)?;

        if tt != variable.variable_type() {
            bail!(ErrorKind::UniformDeclarationMismatch);
        }

        for pair in &mut self.uniforms {
            if pair.0 == name {
                pair.1 = variable;
                return Ok(());
            }
        }

        self.uniforms.push((name.to_owned(), variable));
        Ok(())
    }
}

impl super::Resource for Material {
    fn size(&self) -> usize {
        0
    }
}