use math;
use utils::data_buf;
use utils::hash_value;

use super::assets::prelude::*;
use super::backends::frame::Command;
use super::errors::*;
use super::service::VideoSystemShared;
use super::MAX_UNIFORM_VARIABLES;

/// `OrderDrawBatch` as the named bucket of draw commands. Drawcalls inside `OrderDrawBatch`
/// are sorted before submitting to underlaying OpenGL.
pub struct OrderDrawBatch<T: Ord + Copy> {
    cmds: Vec<(T, Command)>,
    bufs: data_buf::DataBuffer,
}

impl<T: Ord + Copy> OrderDrawBatch<T> {
    #[inline]
    pub fn new() -> Self {
        OrderDrawBatch {
            cmds: Vec::with_capacity(32),
            bufs: data_buf::DataBuffer::with_capacity(512),
        }
    }

    /// Draws ur mesh.
    #[inline]
    pub fn draw(&mut self, order: T, dc: DrawCall) {
        let len = dc.uniforms_len;
        let ptr = self.bufs.extend_from_slice(&dc.uniforms[0..len]);
        let cmd = Command::Draw(dc.shader, dc.mesh, dc.mesh_index, ptr);
        self.cmds.push((order, cmd));
    }

    /// Clears the batch, and submits all the sorted commands into video device. Its guaranteed that
    /// all the commands in this batch will be executed one by one in order.
    ///
    /// Notes that this method has no effect on the allocated capacity of the underlying storage.
    pub fn submit(&mut self, video: &VideoSystemShared, surface: SurfaceHandle) -> Result<()> {
        let mut frame = video.frames.front();
        frame.cmds.push(Command::Bind(surface));

        self.cmds.as_mut_slice().sort_by_key(|v| v.0);
        for v in self.cmds.drain(..) {
            match v {
                (_, Command::Draw(shader, mesh, mesh_index, ptr)) => {
                    let vars = self.bufs.as_slice(ptr);
                    let ptr = frame.bufs.extend_from_slice(vars);
                    let cmd = Command::Draw(shader, mesh, mesh_index, ptr);
                    frame.cmds.push(cmd);
                }

                _ => {}
            }
        }

        self.bufs.clear();
        Ok(())
    }
}

/// In case where order has to be preserved (for example in rendering GUIs), view can be set to
/// be in sequential order with `Batch`. Sequential order is less efficient, because it
/// doesn't allow state change optimization, and should be avoided when possible.
pub struct Batch {
    cmds: Vec<Command>,
    bufs: data_buf::DataBuffer,
}

impl Batch {
    /// Creates a new and empty `Batch`.
    ///
    /// Batch will be cleared
    #[inline]
    pub fn new() -> Self {
        Batch {
            cmds: Vec::with_capacity(32),
            bufs: data_buf::DataBuffer::with_capacity(512),
        }
    }

    /// Draws ur mesh.
    #[inline]
    pub fn draw(&mut self, dc: DrawCall) {
        let len = dc.uniforms_len;
        let ptr = self.bufs.extend_from_slice(&dc.uniforms[0..len]);
        let cmd = Command::Draw(dc.shader, dc.mesh, dc.mesh_index, ptr);
        self.cmds.push(cmd);
    }

    /// Updates the scissor test of surface.
    ///
    /// The test is initially disabled. While the test is enabled, only pixels that lie within
    /// the scissor box can be modified by drawing commands.
    #[inline]
    pub fn update_scissor(&mut self, scissor: SurfaceScissor) {
        self.cmds.push(Command::UpdateScissor(scissor));
    }

    /// Updates the viewport of surface.
    #[inline]
    pub fn update_viewport(&mut self, viewport: SurfaceViewport) {
        self.cmds.push(Command::UpdateViewport(viewport));
    }

    /// Update a contiguous subregion of an existing two-dimensional texture object.
    #[inline]
    pub fn update_texture(&mut self, id: TextureHandle, area: math::Aabb2<u32>, bytes: &[u8]) {
        let bufs = &mut self.bufs;
        let ptr = bufs.extend_from_slice(bytes);
        self.cmds.push(Command::UpdateTexture(id, area, ptr));
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    #[inline]
    pub fn update_vertex_buffer(&mut self, id: MeshHandle, offset: usize, bytes: &[u8]) {
        let bufs = &mut self.bufs;
        let ptr = bufs.extend_from_slice(bytes);
        self.cmds.push(Command::UpdateVertexBuffer(id, offset, ptr));
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    #[inline]
    pub fn update_index_buffer(&mut self, id: MeshHandle, offset: usize, bytes: &[u8]) {
        let bufs = &mut self.bufs;
        let ptr = bufs.extend_from_slice(bytes);
        self.cmds.push(Command::UpdateIndexBuffer(id, offset, ptr));
    }

    /// Clears the batch, and submits all the commands into video device. Its guaranteed that
    /// all the commands in this batch will be executed one by one in order.
    ///
    /// Notes that this method has no effect on the allocated capacity of the underlying storage.
    pub fn submit(&mut self, video: &VideoSystemShared, surface: SurfaceHandle) -> Result<()> {
        let mut frame = video.frames.front();

        frame.cmds.push(Command::Bind(surface));

        for v in self.cmds.drain(..) {
            match v {
                Command::Draw(shader, mesh, mesh_index, ptr) => {
                    let vars = self.bufs.as_slice(ptr);
                    let ptr = frame.bufs.extend_from_slice(vars);
                    let cmd = Command::Draw(shader, mesh, mesh_index, ptr);
                    frame.cmds.push(cmd);
                }

                Command::UpdateTexture(id, area, ptr) => {
                    let ptr = frame.bufs.extend_from_slice(self.bufs.as_slice(ptr));
                    frame.cmds.push(Command::UpdateTexture(id, area, ptr));
                }

                Command::UpdateVertexBuffer(id, offset, ptr) => {
                    let ptr = frame.bufs.extend_from_slice(self.bufs.as_slice(ptr));
                    let cmd = Command::UpdateVertexBuffer(id, offset, ptr);
                    frame.cmds.push(cmd);
                }

                Command::UpdateIndexBuffer(id, offset, ptr) => {
                    let ptr = frame.bufs.extend_from_slice(self.bufs.as_slice(ptr));
                    frame.cmds.push(Command::UpdateIndexBuffer(id, offset, ptr));
                }

                other => frame.cmds.push(other),
            }
        }

        self.bufs.clear();
        Ok(())
    }
}

/// A draw call.
#[derive(Debug, Copy, Clone)]
pub struct DrawCall {
    pub(crate) uniforms: [(hash_value::HashValue<str>, UniformVariable); MAX_UNIFORM_VARIABLES],
    pub(crate) uniforms_len: usize,

    pub shader: ShaderHandle,
    pub mesh: MeshHandle,
    pub mesh_index: MeshIndex,
}

impl DrawCall {
    /// Creates a new and empty draw call.
    pub fn new(shader: ShaderHandle, mesh: MeshHandle) -> Self {
        let nil = (hash_value::HashValue::zero(), UniformVariable::I32(0));
        DrawCall {
            shader: shader,
            uniforms: [nil; MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
            mesh: mesh,
            mesh_index: MeshIndex::All,
        }
    }

    /// Binds the named field with `UniformVariable`.
    pub fn set_uniform_variable<F, V>(&mut self, field: F, variable: V)
    where
        F: Into<hash_value::HashValue<str>>,
        V: Into<UniformVariable>,
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
}
