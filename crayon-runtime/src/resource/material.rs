use math;
use graphics::UniformVariable;
use super::{TexturePtr, ShaderPtr};

/// `Material` exposes all properties from a shader and allowing you to acess them.
#[derive(Debug, Clone)]
pub struct Material {
    shader: ShaderPtr,
    textures: Vec<(String, Option<TexturePtr>)>,
    uniforms: Vec<(String, UniformVariable)>,
    priority: i32,
}

macro_rules! set_uniform_mat {
    ($name: ident, $variable: ident, $input: ident) => (
        pub fn $name(&mut self, name: &str, matrix: &math::$input<f32>, transpose: bool) {
            if let Some(pair) = self.uniform_mut(name) {
                if let &mut (_, UniformVariable::$variable(_, _)) = pair {
                    pair.1 = UniformVariable::$variable(*matrix.as_ref(), transpose);
                }
            }
        }
    )
}

macro_rules! set_uniform_vec {
    ($name: ident, $variable: ident, $input: ident) => (
        pub fn $name(&mut self, name: &str, vec: &math::$input<f32>) {
            if let Some(pair) = self.uniform_mut(name) {
                if let &mut (_, UniformVariable::$variable(_)) = pair {
                    pair.1 = UniformVariable::$variable(*vec.as_ref());
                }
            }
        }
    )
}

impl Material {
    pub fn new(shader: ShaderPtr,
               textures: Vec<(String, Option<TexturePtr>)>,
               uniforms: Vec<(String, UniformVariable)>)
               -> Material {
        Material {
            shader: shader,
            textures: textures,
            uniforms: uniforms,
            priority: 0,
        }
    }

    /// Render queue of this material, renderer will draw objects in order.
    pub fn render_order(&self) -> i32 {
        self.priority
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
    pub fn set_texture(&mut self, name: &str, tex: Option<TexturePtr>) {
        for pair in &mut self.textures {
            if pair.0 == name {
                pair.1 = tex;
                break;
            }
        }
    }

    /// Sets a named matrix uniform.
    set_uniform_mat!(set_matrix2f, Matrix2f, Matrix2);
    set_uniform_mat!(set_matrix3f, Matrix4f, Matrix4);
    set_uniform_mat!(set_matrix4f, Matrix4f, Matrix4);

    /// Sets a named vector uniform.
    set_uniform_vec!(set_vector2f, Vector2f, Vector2);
    set_uniform_vec!(set_vector3f, Vector3f, Vector3);
    set_uniform_vec!(set_vector4f, Vector4f, Vector4);

    fn uniform_mut(&mut self, name: &str) -> Option<&mut (String, UniformVariable)> {
        for pair in &mut self.uniforms {
            if pair.0 == name {
                return Some(pair);
            }
        }

        None
    }
}

impl super::Resource for Material {
    fn size(&self) -> usize {
        0
    }
}