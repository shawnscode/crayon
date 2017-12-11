use super::*;
use super::errors::*;

use utils::HashValue;

/// A draw call.
#[derive(Debug, Copy, Clone)]
pub struct DrawCall {
    order: u64,
    vso: ViewStateHandle,
    shader: ShaderHandle,
    uniforms: [(HashValue<str>, UniformVariable); MAX_UNIFORM_VARIABLES],
    uniforms_len: usize,
    vbo: Option<VertexBufferHandle>,
    ibo: Option<IndexBufferHandle>,
}

impl DrawCall {
    /// Create a new ane empty draw call.
    pub fn new(vso: ViewStateHandle, shader: ShaderHandle) -> Self {
        DrawCall {
            order: 0,
            vso: vso,
            shader: shader,
            uniforms: [(HashValue::zero(), UniformVariable::I32(0)); MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
            vbo: None,
            ibo: None,
        }
    }

    pub fn from(vso: ViewStateHandle, mat: &Material) -> Self {
        let mut dc = DrawCall {
            order: 0,
            vso: vso,
            shader: mat.shader(),
            uniforms: [(HashValue::zero(), UniformVariable::I32(0)); MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
            vbo: None,
            ibo: None,
        };

        for &(field, variable) in mat.iter() {
            dc.set_uniform_variable(field, variable);
        }

        dc
    }

    /// Bind vertex buffer and optional index buffer.
    pub fn set_mesh<T>(&mut self, vbo: VertexBufferHandle, ibo: T)
        where T: Into<Option<IndexBufferHandle>>
    {
        self.vbo = Some(vbo);
        self.ibo = ibo.into();
    }

    /// Bind the named field with `UniformVariable`.
    pub fn set_uniform_variable<F, T>(&mut self, field: F, variable: T)
        where F: Into<HashValue<str>>,
              T: Into<UniformVariable>
    {
        assert!(self.uniforms_len < MAX_UNIFORM_VARIABLES);

        let field = field.into();
        let variable = variable.into();

        for i in 0..self.uniforms_len {
            if self.uniforms[i].0 == field {
                self.uniforms[i] = (field, variable);
                return;
            }
        }

        self.uniforms[self.uniforms_len] = (field, variable);
        self.uniforms_len += 1;
    }

    /// Submit primitive for drawing, within view all draw commands are executed after
    /// resource manipulation, such like `create_vertex_buffer`, `update_vertex_buffer`,
    /// etc.
    pub fn submit(&self,
                  video: &GraphicsSystemShared,
                  primitite: Primitive,
                  from: u32,
                  len: u32)
                  -> Result<()> {
        let vbo = self.vbo.ok_or(ErrorKind::CanNotDrawWihtoutVertexBuffer)?;

        video.submit(self.order,
                     self.vso,
                     self.shader,
                     &self.uniforms[0..self.uniforms_len],
                     vbo,
                     self.ibo,
                     primitite,
                     from,
                     len)
    }
}
