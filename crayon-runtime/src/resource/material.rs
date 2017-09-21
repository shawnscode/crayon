use std::collections::HashMap;
use graphics;
use graphics::{UniformVariable, UniformVariableType};

use super::errors::*;
use super::{TexturePtr, ShaderPtr};

/// `Material` exposes all properties from a shader and allowing you to acess them.
#[derive(Debug, Clone)]
pub struct Material {
    shader: ShaderPtr,
    textures: HashMap<String, Option<TexturePtr>>,
    uniforms: HashMap<String, UniformVariable>,
    order: i32,
}

impl Material {
    pub fn new(shader: ShaderPtr) -> Material {
        Material {
            shader: shader,
            textures: HashMap::new(),
            uniforms: HashMap::new(),
            order: 0,
        }
    }

    /// Get underlying shader of this material.
    pub fn shader(&self) -> &ShaderPtr {
        &self.shader
    }

    /// Render queue of this material, renderer will draw objects in order.
    #[inline]
    pub fn order(&self) -> i32 {
        self.order
    }

    /// Get the uniform variable with given name.
    #[inline]
    pub fn uniform_variable(&self, name: &str) -> Option<UniformVariable> {
        self.uniforms.get(name).map(|v| *v)
    }

    /// Get the texture with given name.
    #[inline]
    pub fn texture(&self, name: &str) -> Option<TexturePtr> {
        self.textures.get(name).map(|v| v.clone()).unwrap_or(None)
    }

    /// Return true if we have a uniform variable with specified type.
    #[inline]
    pub fn has_uniform_variable(&self, name: &str, lhs: UniformVariableType) -> bool {
        if let Some(rhs) = self.shader.read().unwrap().uniform_variable(name) {
            lhs == rhs
        } else {
            false
        }
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

        self.textures.insert(name.to_owned(), texture);
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

        self.uniforms.insert(name.to_owned(), variable);
        Ok(())
    }

    pub fn update_video_object(&mut self,
                               mut video: &mut graphics::Graphics)
                               -> graphics::errors::Result<()> {
        for (_, v) in &self.textures {
            if let &Some(ref texture) = v {
                let mut texture = texture.write().unwrap();
                texture.update_video_object(&mut video)?;
            }
        }

        Ok(())
    }

    pub fn extract(&self, task: &mut graphics::FrameTaskBuilder) {
        for (name, v) in &self.uniforms {
            task.with_uniform_variable(&name, *v);
        }

        for (name, v) in &self.textures {
            if let &Some(ref texture) = v {
                let texture = texture.write().unwrap();
                if let Some(handle) = texture.video_object() {
                    task.with_texture(&name, handle);
                }
            }
        }
    }
}

impl super::Resource for Material {
    fn size(&self) -> usize {
        0
    }
}