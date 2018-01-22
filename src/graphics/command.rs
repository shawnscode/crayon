use super::*;
use super::errors::*;

use utils::{HashValue, Rect};

/// `Command` will be executed in sequential order.
pub enum Command<'a> {
    DrawCall(SliceDrawCall<'a>),
    VertexBufferUpdate(VertexBufferUpdate<'a>),
    IndexBufferUpdate(IndexBufferUpdate<'a>),
    TextureUpdate(TextureUpdate<'a>),
    SetScissor(ScissorUpdate),
}

impl<'a> Command<'a> {
    pub fn update_vertex_buffer(mesh: MeshHandle, offset: usize, data: &[u8]) -> Command {
        let task = VertexBufferUpdate {
            mesh: mesh,
            offset: offset,
            data: data,
        };

        Command::VertexBufferUpdate(task)
    }

    pub fn update_index_buffer(mesh: MeshHandle, offset: usize, data: &[u8]) -> Command {
        let task = IndexBufferUpdate {
            mesh: mesh,
            offset: offset,
            data: data,
        };

        Command::IndexBufferUpdate(task)
    }

    pub fn update_texture(texture: TextureHandle, rect: Rect, data: &[u8]) -> Command {
        let task = TextureUpdate {
            texture: texture,
            rect: rect,
            data: data,
        };

        Command::TextureUpdate(task)
    }

    pub fn set_scissor(scissor: Scissor) -> Command<'a> {
        Command::SetScissor(ScissorUpdate { scissor: scissor })
    }
}

/// Draw.
pub struct SliceDrawCall<'a> {
    pub(crate) shader: ShaderHandle,
    pub(crate) uniforms: &'a [(HashValue<str>, UniformVariable)],
    pub(crate) mesh: MeshHandle,
    pub(crate) index: MeshIndex,
}

impl<'a> Into<Command<'a>> for SliceDrawCall<'a> {
    fn into(self) -> Command<'a> {
        Command::DrawCall(self)
    }
}

/// Vertex buffer object update.
pub struct VertexBufferUpdate<'a> {
    pub(crate) mesh: MeshHandle,
    pub(crate) offset: usize,
    pub(crate) data: &'a [u8],
}

/// Index buffer object update.
pub struct IndexBufferUpdate<'a> {
    pub(crate) mesh: MeshHandle,
    pub(crate) offset: usize,
    pub(crate) data: &'a [u8],
}

/// Texture object update.
pub struct TextureUpdate<'a> {
    pub(crate) texture: TextureHandle,
    pub(crate) rect: Rect,
    pub(crate) data: &'a [u8],
}

/// Scissor update.
pub struct ScissorUpdate {
    pub(crate) scissor: Scissor,
}

/// A draw call.
#[derive(Debug, Copy, Clone)]
pub struct DrawCall {
    shader: ShaderHandle,
    uniforms: [(HashValue<str>, UniformVariable); MAX_UNIFORM_VARIABLES],
    uniforms_len: usize,
    mesh: MeshHandle,
}

impl DrawCall {
    /// Create a new and empty draw call.
    pub fn new(shader: ShaderHandle, mesh: MeshHandle) -> Self {
        DrawCall {
            shader: shader,
            uniforms: [(HashValue::zero(), UniformVariable::I32(0)); MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
            mesh: mesh,
        }
    }

    /// Creates a new `DrawCall` from material.
    pub fn from(mat: &Material, mesh: MeshHandle) -> Self {
        let mut dc = DrawCall {
            shader: mat.shader(),
            uniforms: [(HashValue::zero(), UniformVariable::I32(0)); MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
            mesh: mesh,
        };

        for &(field, variable) in mat.iter() {
            dc.set_uniform_variable(field, variable);
        }

        dc
    }

    /// Bind the named field with `UniformVariable`.
    pub fn set_uniform_variable<F, T>(&mut self, field: F, variable: T)
    where
        F: Into<HashValue<str>>,
        T: Into<UniformVariable>,
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

    pub fn build(&mut self, index: MeshIndex) -> Result<SliceDrawCall> {
        let task = SliceDrawCall {
            shader: self.shader,
            uniforms: &self.uniforms[0..self.uniforms_len],
            mesh: self.mesh,
            index: index,
        };

        Ok(task)
    }

    pub fn build_from(&mut self, from: usize, len: usize) -> Result<SliceDrawCall> {
        let task = SliceDrawCall {
            shader: self.shader,
            uniforms: &self.uniforms[0..self.uniforms_len],
            mesh: self.mesh,
            index: MeshIndex::Ptr(from, len),
        };

        Ok(task)
    }

    pub fn build_sub_mesh(&mut self, index: usize) -> Result<SliceDrawCall> {
        let task = SliceDrawCall {
            shader: self.shader,
            uniforms: &self.uniforms[0..self.uniforms_len],
            mesh: self.mesh,
            index: MeshIndex::SubMesh(index),
        };

        Ok(task)
    }
}
