use crate::math::prelude::Aabb2;
use crate::utils::prelude::{DataBuffer, HashValue};

use super::assets::prelude::*;
use super::backends::frame::Command;
use super::errors::*;
use super::MAX_UNIFORM_VARIABLES;

/// The command buffer of video system.
#[derive(Default)]
pub struct CommandBuffer {
    cmds: Vec<Command>,
    bufs: DataBuffer,
}

impl CommandBuffer {
    /// Creates a new and empty `CommandBuffer`.
    #[inline]
    pub fn new() -> Self {
        CommandBuffer {
            cmds: Vec::with_capacity(32),
            bufs: DataBuffer::with_capacity(512),
        }
    }

    /// Draws ur mesh.
    #[inline]
    pub fn draw(&mut self, dc: Draw) {
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
    pub fn update_texture(&mut self, id: TextureHandle, area: Aabb2<u32>, bytes: &[u8]) {
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
    pub fn submit(&mut self, surface: SurfaceHandle) -> Result<()> {
        let doubele_frame = unsafe { super::frames() };
        let mut frame = doubele_frame.write();
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

/// The draw call buffer of video system, which provides simple sort functionality for convenience.
pub struct DrawCommandBuffer<T: Ord + Copy> {
    cmds: Vec<(T, Command)>,
    bufs: DataBuffer,
}

impl<T: Ord + Copy> Default for DrawCommandBuffer<T> {
    fn default() -> Self {
        DrawCommandBuffer {
            cmds: Vec::with_capacity(32),
            bufs: DataBuffer::with_capacity(512),
        }
    }
}

impl<T: Ord + Copy> DrawCommandBuffer<T> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Draws ur mesh.
    #[inline]
    pub fn draw(&mut self, order: T, dc: Draw) {
        let len = dc.uniforms_len;
        let ptr = self.bufs.extend_from_slice(&dc.uniforms[0..len]);
        let cmd = Command::Draw(dc.shader, dc.mesh, dc.mesh_index, ptr);
        self.cmds.push((order, cmd));
    }

    /// Clears the batch, and submits all the sorted commands into video device. Its guaranteed that
    /// all the commands in this batch will be executed one by one in order.
    ///
    /// Notes that this method has no effect on the allocated capacity of the underlying storage.
    pub fn submit(&mut self, surface: SurfaceHandle) -> Result<()> {
        let doubele_frame = unsafe { super::frames() };
        let mut frame = doubele_frame.write();
        frame.cmds.push(Command::Bind(surface));

        self.cmds.as_mut_slice().sort_by_key(|v| v.0);
        for v in self.cmds.drain(..) {
            if let (_, Command::Draw(shader, mesh, mesh_index, ptr)) = v {
                let vars = self.bufs.as_slice(ptr);
                let ptr = frame.bufs.extend_from_slice(vars);
                let cmd = Command::Draw(shader, mesh, mesh_index, ptr);
                frame.cmds.push(cmd);
            }
        }

        self.bufs.clear();
        Ok(())
    }
}

/// A draw call.
#[derive(Debug, Copy, Clone)]
pub struct Draw {
    pub(crate) uniforms: [(HashValue<str>, UniformVariable); MAX_UNIFORM_VARIABLES],
    pub(crate) uniforms_len: usize,

    pub shader: ShaderHandle,
    pub mesh: MeshHandle,
    pub mesh_index: MeshIndex,
}

impl Draw {
    /// Creates a new and empty draw call.
    pub fn new(shader: ShaderHandle, mesh: MeshHandle) -> Self {
        let nil = (HashValue::zero(), UniformVariable::I32(0));
        Draw {
            shader,
            mesh,
            uniforms: [nil; MAX_UNIFORM_VARIABLES],
            uniforms_len: 0,
            mesh_index: MeshIndex::All,
        }
    }

    /// Binds the named field with `UniformVariable`.
    pub fn set_uniform_variable<F, V>(&mut self, field: F, variable: V)
    where
        F: Into<HashValue<str>>,
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
