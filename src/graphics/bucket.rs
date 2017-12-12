use super::*;
use super::errors::*;

use utils::{HashValue, Rect};

/// `BucketTask` will be executed in sequential order.
pub enum BucketTask<'a> {
    DrawCall(BucketDrawCall<'a>),
    VertexBufferUpdate(BucketVertexBufferUpdate<'a>),
    IndexBufferUpdate(BucketIndexBufferUpdate<'a>),
    TextureUpdate(BucketTextureUpdate<'a>),
    SetScissor(BucketScissorUpdate),
}

impl<'a> BucketTask<'a> {
    pub fn update_vertex_buffer(vbo: VertexBufferHandle, offset: usize, data: &[u8]) -> BucketTask {
        let task = BucketVertexBufferUpdate {
            vbo: vbo,
            offset: offset,
            data: data,
        };

        BucketTask::VertexBufferUpdate(task)
    }

    pub fn update_index_buffer(ibo: IndexBufferHandle, offset: usize, data: &[u8]) -> BucketTask {
        let task = BucketIndexBufferUpdate {
            ibo: ibo,
            offset: offset,
            data: data,
        };

        BucketTask::IndexBufferUpdate(task)
    }

    pub fn update_texture(texture: TextureHandle, rect: Rect, data: &[u8]) -> BucketTask {
        let task = BucketTextureUpdate {
            texture: texture,
            rect: rect,
            data: data,
        };

        BucketTask::TextureUpdate(task)
    }

    pub fn set_scissor(scissor: Scissor) -> BucketTask<'a> {
        BucketTask::SetScissor(BucketScissorUpdate { scissor: scissor })
    }
}

/// Draw.
pub struct BucketDrawCall<'a> {
    pub(crate) shader: ShaderHandle,
    pub(crate) uniforms: &'a [(HashValue<str>, UniformVariable)],
    pub(crate) vbo: VertexBufferHandle,
    pub(crate) ibo: Option<IndexBufferHandle>,
    pub(crate) primitive: Primitive,
    pub(crate) from: u32,
    pub(crate) len: u32,
}

impl<'a> Into<BucketTask<'a>> for BucketDrawCall<'a> {
    fn into(self) -> BucketTask<'a> {
        BucketTask::DrawCall(self)
    }
}

/// Vertex buffer object update.
pub struct BucketVertexBufferUpdate<'a> {
    pub(crate) vbo: VertexBufferHandle,
    pub(crate) offset: usize,
    pub(crate) data: &'a [u8],
}

/// Index buffer object update.
pub struct BucketIndexBufferUpdate<'a> {
    pub(crate) ibo: IndexBufferHandle,
    pub(crate) offset: usize,
    pub(crate) data: &'a [u8],
}

/// Texture object update.
pub struct BucketTextureUpdate<'a> {
    pub(crate) texture: TextureHandle,
    pub(crate) rect: Rect,
    pub(crate) data: &'a [u8],
}

/// Scissor update.
pub struct BucketScissorUpdate {
    pub(crate) scissor: Scissor,
}

/// A draw call.
#[derive(Debug, Copy, Clone)]
pub struct DrawCall {
    shader: ShaderHandle,
    uniforms: [(HashValue<str>, UniformVariable); MAX_UNIFORM_VARIABLES],
    uniforms_len: usize,
    vbo: Option<VertexBufferHandle>,
    ibo: Option<IndexBufferHandle>,
}

impl DrawCall {
    /// Create a new and empty draw call.
    pub fn new(shader: ShaderHandle) -> Self {
        DrawCall {
            shader: shader,
            uniforms: [(HashValue::zero(), UniformVariable::I32(0)); MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
            vbo: None,
            ibo: None,
        }
    }

    /// Creates a new `DrawCall` from material.
    pub fn from(mat: &Material) -> Self {
        let mut dc = DrawCall {
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

    /// Bind vertex buffer and optional index buffer.
    pub fn set_mesh<T>(&mut self, vbo: VertexBufferHandle, ibo: T)
        where T: Into<Option<IndexBufferHandle>>
    {
        self.vbo = Some(vbo);
        self.ibo = ibo.into();
    }

    pub fn draw(&mut self, primitive: Primitive, from: u32, len: u32) -> Result<BucketDrawCall> {
        let vbo = self.vbo.ok_or(ErrorKind::CanNotDrawWihtoutVertexBuffer)?;

        let task = BucketDrawCall {
            shader: self.shader,
            uniforms: &self.uniforms[0..self.uniforms_len],
            vbo: vbo,
            ibo: self.ibo,
            primitive: primitive,
            from: from,
            len: len,
        };

        Ok(task)
    }
}