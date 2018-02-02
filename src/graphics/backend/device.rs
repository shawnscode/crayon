use std::str;
use std::cell::{Cell, RefCell};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;

use gl;
use gl::types::*;

use utils::{Color, DataBuffer, Handle, HashValue, Rect};
use graphics::*;

use super::errors::*;
use super::visitor::*;
use super::frame::{FrameDrawCall, FrameTask};

type ResourceID = GLuint;
type UniformID = GLint;

#[derive(Debug, Clone)]
struct MeshObject {
    vbo: ResourceID,
    ibo: ResourceID,
    setup: MeshSetup,
}

#[derive(Debug)]
struct ShaderObject {
    id: ResourceID,
    render_state: RenderState,
    layout: AttributeLayout,
    uniform_locations: HashMap<HashValue<str>, UniformID>,
    uniforms: HashMap<String, UniformVariable>,
}

#[derive(Debug, Clone)]
struct SurfaceObject {
    setup: SurfaceSetup,
}

#[derive(Debug, Copy, Clone)]
struct TextureObject {
    id: ResourceID,
    setup: TextureSetup,
}

#[derive(Debug, Copy, Clone)]
struct RenderTextureObject {
    id: ResourceID,
    setup: RenderTextureSetup,
}

#[derive(Debug, Copy, Clone)]
struct RenderBufferObject {
    id: ResourceID,
    setup: RenderBufferSetup,
}

#[derive(Debug, Copy, Clone)]
struct FrameBufferObject {
    id: ResourceID,
    dimensions: Option<(u16, u16)>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct FrameInfo {
    pub drawcall: u32,
    pub triangles: u32,
}

pub(crate) struct Device {
    visitor: OpenGLVisitor,

    meshes: DataVec<MeshObject>,
    shaders: DataVec<ShaderObject>,
    surfaces: DataVec<SurfaceObject>,
    textures: DataVec<TextureObject>,
    render_textures: DataVec<RenderTextureObject>,
    render_buffers: DataVec<RenderBufferObject>,
    framebuffers: DataVec<FrameBufferObject>,

    active_shader: Cell<Option<ShaderHandle>>,
    frame_info: RefCell<FrameInfo>,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Device {
    pub unsafe fn new() -> Self {
        Device {
            visitor: OpenGLVisitor::new(),
            meshes: DataVec::new(),
            shaders: DataVec::new(),
            surfaces: DataVec::new(),
            textures: DataVec::new(),
            render_buffers: DataVec::new(),
            render_textures: DataVec::new(),
            framebuffers: DataVec::new(),
            active_shader: Cell::new(None),
            frame_info: RefCell::new(FrameInfo::default()),
        }
    }
}

impl Device {
    pub unsafe fn run_one_frame(&self) -> Result<()> {
        self.active_shader.set(None);
        self.visitor.bind_framebuffer(0, false)?;
        self.visitor.clear(Color::black(), None, None)?;
        self.visitor.set_scissor(Scissor::Disable)?;

        *self.frame_info.borrow_mut() = FrameInfo::default();
        Ok(())
    }

    pub fn frame_info(&self) -> FrameInfo {
        *self.frame_info.borrow()
    }

    pub fn flush(
        &mut self,
        tasks: &mut [(SurfaceHandle, u64, FrameTask)],
        buf: &DataBuffer,
        dimensions: (u32, u32),
        hidpi: f32,
    ) -> Result<()> {
        // Sort frame tasks by user defined priorities. Notes that Slice::sort_by
        // is stable, which means it does not reorder equal elements, so it will
        // not change the execution order in one specific surface.
        tasks.sort_by(|lhs, rhs| {
            let lv = self.surfaces.get(lhs.0).unwrap();
            let rv = self.surfaces.get(rhs.0).unwrap();
            let mut ord = lv.setup.order.cmp(&rv.setup.order);

            if ord == Ordering::Equal && !lv.setup.sequence {
                ord = lhs.1.cmp(&rhs.1);
            }

            ord
        });

        let dimensions = (dimensions.0 as u16, dimensions.1 as u16);
        unsafe {
            // Submit real OpenGL drawcall in order.
            let mut surface = None;
            for v in tasks {
                if surface != Some(v.0) {
                    surface = Some(v.0);
                    self.rebind_surface(v.0, dimensions, hidpi)?;
                }

                match v.2 {
                    FrameTask::DrawCall(dc) => self.draw(dc, buf)?,

                    FrameTask::UpdateSurface(scissor) => self.visitor.set_scissor(scissor)?,

                    FrameTask::UpdateVertexBuffer(vbo, offset, ptr) => {
                        let data = buf.as_slice(ptr);
                        self.update_vertex_buffer(vbo, offset, data)?;
                    }

                    FrameTask::UpdateIndexBuffer(ibo, offset, ptr) => {
                        let data = buf.as_slice(ptr);
                        self.update_index_buffer(ibo, offset, data)?;
                    }

                    FrameTask::UpdateTexture(texture, rect, ptr) => {
                        let data = buf.as_slice(ptr);
                        self.update_texture(texture, rect, data)?;
                    }
                }
            }

            self.visitor.flush()?;
        }

        Ok(())
    }

    unsafe fn draw(&self, dc: FrameDrawCall, buf: &DataBuffer) -> Result<()> {
        // Bind program and associated uniforms and textures.
        let shader = self.bind_shader(dc.shader)?;

        let mut texture_idx = 0;
        for &(field, ptr) in buf.as_slice(dc.uniforms) {
            let variable = buf.as_ref(ptr);
            let location = shader.uniform_locations[&field];

            match *variable {
                UniformVariable::Texture(handle) => {
                    if let Some(texture) = self.textures.get(handle) {
                        let v = UniformVariable::I32(texture_idx);
                        self.visitor.bind_uniform(location, &v)?;
                        self.visitor.bind_texture(texture_idx as u32, texture.id)?;
                        texture_idx += 1;
                    }
                }
                UniformVariable::RenderTexture(handle) => {
                    if let Some(texture) = self.render_textures.get(handle) {
                        let v = UniformVariable::I32(texture_idx);
                        self.visitor.bind_uniform(location, &v)?;
                        self.visitor.bind_texture(texture_idx as u32, texture.id)?;
                        texture_idx += 1;
                    }
                }
                _ => {
                    self.visitor.bind_uniform(location, variable)?;
                }
            }
        }

        // Bind vertex buffer and vertex array object.
        let mesh = self.meshes.get(dc.mesh).ok_or(ErrorKind::InvalidHandle)?;
        self.visitor.bind_buffer(gl::ARRAY_BUFFER, mesh.vbo)?;
        self.visitor
            .bind_attribute_layout(&shader.layout, &mesh.setup.layout)?;

        // Bind index buffer object if available.
        self.visitor
            .bind_buffer(gl::ELEMENT_ARRAY_BUFFER, mesh.ibo)?;

        let (from, len) = match dc.index {
            MeshIndex::Ptr(from, len) => {
                if (from + len) > mesh.setup.num_idxes {
                    bail!("Invalid index of sub-mesh!");
                }

                (
                    (from * mesh.setup.index_format.stride()) as u32,
                    len as GLsizei,
                )
            }
            MeshIndex::SubMesh(index) => {
                let num = mesh.setup.sub_mesh_offsets.len();
                if index >= num || num == 0 {
                    bail!("Invalid index of sub-mesh!");
                }

                let from = mesh.setup.sub_mesh_offsets[index];
                let to = if index == (num - 1) {
                    mesh.setup.num_idxes
                } else {
                    mesh.setup.sub_mesh_offsets[index + 1]
                };

                (
                    (from * mesh.setup.index_format.stride()) as u32,
                    (to - from) as GLsizei,
                )
            }
            MeshIndex::All => (0, mesh.setup.num_idxes as i32),
        };

        gl::DrawElements(
            mesh.setup.primitive.into(),
            len,
            mesh.setup.index_format.into(),
            from as *const u32 as *const ::std::os::raw::c_void,
        );

        {
            let mut info = self.frame_info.borrow_mut();
            info.drawcall += 1;
            info.triangles += mesh.setup.primitive.assemble_triangles(len as u32);
        }

        check()
    }

    unsafe fn rebind_surface(
        &self,
        handle: SurfaceHandle,
        dimensions: (u16, u16),
        hidpi: f32,
    ) -> Result<()> {
        let surface = self.surfaces.get(handle).ok_or(ErrorKind::InvalidHandle)?;
        let dimensions = (
            (f32::from(dimensions.0) * hidpi) as u16,
            (f32::from(dimensions.1) * hidpi) as u16,
        );

        // Bind frame buffer.
        let dimensions = if let Some(fbo) = surface.setup.framebuffer {
            if let Some(fbo) = self.framebuffers.get(fbo) {
                self.visitor.bind_framebuffer(fbo.id, true)?;
                fbo.dimensions.unwrap_or(dimensions)
            } else {
                bail!(ErrorKind::InvalidHandle);
            }
        } else {
            self.visitor.bind_framebuffer(0, false)?;
            dimensions
        };

        let vp = surface.setup.viewport;
        let position = (
            ((vp.0).0 * f32::from(dimensions.0)) as u16,
            ((vp.0).1 * f32::from(dimensions.1)) as u16,
        );
        let dimensions = (
            ((vp.1).0 * f32::from(dimensions.0)) as u16,
            ((vp.1).1 * f32::from(dimensions.1)) as u16,
        );

        // Binds the viewport and scissor box.
        self.visitor.set_viewport(position, dimensions)?;
        self.visitor.set_scissor(Scissor::Disable)?;
        // Sets depth write enable to make sure that we can clear depth buffer properly.
        self.visitor.set_depth_write(true, None)?;

        // Clears frame buffer.
        self.visitor.clear(
            surface.setup.clear_color,
            surface.setup.clear_depth,
            surface.setup.clear_stencil,
        )?;

        Ok(())
    }

    unsafe fn bind_shader(&self, handle: ShaderHandle) -> Result<&ShaderObject> {
        let shader = self.shaders.get(handle).ok_or(ErrorKind::InvalidHandle)?;

        if let Some(v) = self.active_shader.get() {
            if v == handle {
                return Ok(shader);
            }
        }

        self.visitor.bind_program(shader.id)?;

        let state = &shader.render_state;
        self.visitor.set_cull_face(state.cull_face)?;
        self.visitor.set_front_face_order(state.front_face_order)?;
        self.visitor.set_depth_test(state.depth_test)?;
        self.visitor
            .set_depth_write(state.depth_write, state.depth_write_offset)?;
        self.visitor.set_color_blend(state.color_blend)?;

        let c = &state.color_write;
        self.visitor.set_color_write(c.0, c.1, c.2, c.3)?;

        for (name, variable) in &shader.uniforms {
            let location = self.visitor.get_uniform_location(shader.id, name)?;
            if location != -1 {
                self.visitor.bind_uniform(location, variable)?;
            }
        }

        self.active_shader.set(Some(handle));
        Ok(shader)
    }
}

impl Device {
    pub unsafe fn create_mesh(
        &mut self,
        handle: MeshHandle,
        setup: MeshSetup,
        verts: Option<&[u8]>,
        idxes: Option<&[u8]>,
    ) -> Result<()> {
        if self.meshes.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let vbo = self.visitor.create_buffer(
            OpenGLBuffer::Vertex,
            setup.hint,
            setup.vertex_buffer_len() as u32,
            verts,
        )?;

        let ibo = self.visitor.create_buffer(
            OpenGLBuffer::Index,
            setup.hint,
            setup.index_buffer_len() as u32,
            idxes,
        )?;

        let mesh = MeshObject {
            vbo: vbo,
            ibo: ibo,
            setup: setup,
        };

        self.meshes.set(handle, mesh);
        check()
    }

    pub unsafe fn update_vertex_buffer(
        &mut self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        if let Some(mesh) = self.meshes.get(handle) {
            if mesh.setup.hint == BufferHint::Immutable {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() + offset > mesh.setup.vertex_buffer_len() {
                bail!(ErrorKind::OutOfBounds);
            }

            self.visitor
                .update_buffer(mesh.vbo, OpenGLBuffer::Vertex, offset as u32, data)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn update_index_buffer(
        &mut self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        if let Some(mesh) = self.meshes.get(handle) {
            if mesh.setup.hint == BufferHint::Immutable {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() + offset > mesh.setup.index_buffer_len() {
                bail!(ErrorKind::OutOfBounds);
            }

            self.visitor
                .update_buffer(mesh.ibo, OpenGLBuffer::Index, offset as u32, data)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn delete_mesh(&mut self, handle: MeshHandle) -> Result<()> {
        if let Some(mesh) = self.meshes.remove(handle) {
            self.visitor.delete_buffer(mesh.vbo)?;
            self.visitor.delete_buffer(mesh.ibo)?;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_render_buffer(
        &mut self,
        handle: RenderBufferHandle,
        setup: RenderBufferSetup,
    ) -> Result<()> {
        let (internal_format, _, _) = setup.format.into();
        let id = self.visitor.create_render_buffer(
            internal_format,
            setup.dimensions.0,
            setup.dimensions.1,
        )?;

        self.render_buffers.set(
            handle,
            RenderBufferObject {
                id: id,
                setup: setup,
            },
        );
        Ok(())
    }

    pub unsafe fn delete_render_buffer(&mut self, handle: RenderBufferHandle) -> Result<()> {
        if let Some(rto) = self.render_buffers.remove(handle) {
            self.visitor.delete_render_buffer(rto.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_framebuffer(&mut self, handle: FrameBufferHandle) -> Result<()> {
        if self.framebuffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let fbo = FrameBufferObject {
            id: self.visitor.create_framebuffer()?,
            dimensions: None,
        };

        self.framebuffers.set(handle, fbo);
        Ok(())
    }

    pub unsafe fn update_framebuffer_with_texture(
        &mut self,
        handle: FrameBufferHandle,
        texture: RenderTextureHandle,
        slot: u32,
    ) -> Result<()> {
        let fbo = self.framebuffers
            .get_mut(handle)
            .ok_or(ErrorKind::InvalidHandle)?;

        let rt = self.render_textures
            .get(texture)
            .ok_or(ErrorKind::InvalidHandle)?;

        self.visitor.bind_framebuffer(fbo.id, false)?;

        let attached_dimensions = (rt.setup.dimensions.0 as u16, rt.setup.dimensions.1 as u16);
        if let Some(dimensions) = fbo.dimensions {
            if attached_dimensions != dimensions {
                bail!(
                    "Incompitable(mismatch dimensions) attachment of frame-buffer {:?}",
                    handle
                );
            }
        } else {
            fbo.dimensions = Some(attached_dimensions);
        }

        match rt.setup.format {
            RenderTextureFormat::RGB8 | RenderTextureFormat::RGBA4 | RenderTextureFormat::RGBA8 => {
                let location = gl::COLOR_ATTACHMENT0 + slot;
                self.visitor.bind_framebuffer_with_texture(location, rt.id)
            }
            RenderTextureFormat::Depth16
            | RenderTextureFormat::Depth24
            | RenderTextureFormat::Depth32 => self.visitor
                .bind_framebuffer_with_texture(gl::DEPTH_ATTACHMENT, rt.id),
            RenderTextureFormat::Depth24Stencil8 => self.visitor
                .bind_framebuffer_with_texture(gl::DEPTH_STENCIL_ATTACHMENT, rt.id),
        }
    }

    pub unsafe fn update_framebuffer_with_renderbuffer(
        &mut self,
        handle: FrameBufferHandle,
        buf: RenderBufferHandle,
        slot: u32,
    ) -> Result<()> {
        let fbo = self.framebuffers
            .get(handle)
            .ok_or(ErrorKind::InvalidHandle)?;
        let buf = self.render_buffers
            .get(buf)
            .ok_or(ErrorKind::InvalidHandle)?;

        self.visitor.bind_framebuffer(fbo.id, false)?;
        match buf.setup.format {
            RenderTextureFormat::RGB8 | RenderTextureFormat::RGBA4 | RenderTextureFormat::RGBA8 => {
                let location = gl::COLOR_ATTACHMENT0 + slot;
                self.visitor
                    .bind_framebuffer_with_renderbuffer(location, buf.id)
            }
            RenderTextureFormat::Depth16
            | RenderTextureFormat::Depth24
            | RenderTextureFormat::Depth32 => self.visitor
                .bind_framebuffer_with_renderbuffer(gl::DEPTH_ATTACHMENT, buf.id),
            RenderTextureFormat::Depth24Stencil8 => self.visitor
                .bind_framebuffer_with_renderbuffer(gl::DEPTH_STENCIL_ATTACHMENT, buf.id),
        }
    }

    pub unsafe fn delete_framebuffer(&mut self, handle: FrameBufferHandle) -> Result<()> {
        if let Some(fbo) = self.framebuffers.remove(handle) {
            self.visitor.delete_framebuffer(fbo.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_render_texture(
        &mut self,
        handle: RenderTextureHandle,
        setup: RenderTextureSetup,
    ) -> Result<()> {
        let (internal_format, in_format, pixel_type) = setup.format.into();
        let id = self.visitor.create_texture(
            internal_format,
            in_format,
            pixel_type,
            TextureAddress::Repeat,
            TextureFilter::Linear,
            false,
            setup.dimensions.0,
            setup.dimensions.1,
            None,
        )?;

        self.render_textures.set(
            handle,
            RenderTextureObject {
                id: id,
                setup: setup,
            },
        );
        Ok(())
    }

    pub unsafe fn delete_render_texture(&mut self, handle: RenderTextureHandle) -> Result<()> {
        if let Some(texture) = self.render_textures.remove(handle) {
            self.visitor.delete_texture(texture.id)?;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_texture(
        &mut self,
        handle: TextureHandle,
        setup: TextureSetup,
        data: Option<&[u8]>,
    ) -> Result<()> {
        let (internal_format, in_format, pixel_type) = setup.format.into();
        let id = self.visitor.create_texture(
            internal_format,
            in_format,
            pixel_type,
            setup.address,
            setup.filter,
            setup.mipmap,
            setup.dimensions.0,
            setup.dimensions.1,
            data,
        )?;

        self.textures.set(
            handle,
            TextureObject {
                id: id,
                setup: setup,
            },
        );
        Ok(())
    }

    pub unsafe fn update_texture(
        &mut self,
        handle: TextureHandle,
        rect: Rect,
        data: &[u8],
    ) -> Result<()> {
        if let Some(texture) = self.textures.get(handle) {
            if data.len() > rect.size() as usize || rect.min.x as u32 >= texture.setup.dimensions.0
                || rect.min.y as u32 >= texture.setup.dimensions.1 || rect.max.x < 0
                || rect.max.y < 0
            {
                bail!(ErrorKind::OutOfBounds);
            }

            let (_, format, tt) = texture.setup.format.into();
            self.visitor
                .update_texture(texture.id, format, tt, rect, data)?;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn delete_texture(&mut self, handle: TextureHandle) -> Result<()> {
        if let Some(texture) = self.textures.remove(handle) {
            self.visitor.delete_texture(texture.id)?;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn create_surface(&mut self, handle: SurfaceHandle, setup: SurfaceSetup) -> Result<()> {
        let view = SurfaceObject { setup: setup };
        self.surfaces.set(handle, view);
        Ok(())
    }

    pub fn delete_surface(&mut self, handle: SurfaceHandle) -> Result<()> {
        if self.surfaces.remove(handle).is_some() {
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Initializes named program object. A program object is an object to
    /// which shader objects can be attached. Vertex and fragment shader
    /// are minimal requirement to build a proper program.
    pub unsafe fn create_shader(&mut self, handle: ShaderHandle, setup: ShaderSetup) -> Result<()> {
        let pid = self.visitor.create_program(&setup.vs, &setup.fs)?;

        for (name, _) in setup.layout.iter() {
            let name: &'static str = name.into();
            let location = self.visitor.get_attribute_location(pid, name)?;
            if location == -1 {
                bail!(format!("failed to locate attribute {:?}", name));
            }
        }

        let mut uniform_locations = HashMap::new();
        for (name, _) in setup.uniform_variables {
            let location = self.visitor.get_uniform_location(pid, &name)?;
            if location == -1 {
                bail!(format!("failed to locate uniform {:?}", name));
            }

            uniform_locations.insert(name.into(), location);
        }

        self.shaders.set(
            handle,
            ShaderObject {
                id: pid,
                render_state: setup.render_state,
                layout: setup.layout,
                uniform_locations: uniform_locations,
                uniforms: HashMap::new(),
            },
        );
        check()
    }

    // pub fn update_shader_uniform(&mut self,
    //                                handle: ShaderHandle,
    //                                name: &str,
    //                                variable: &UniformVariable)
    //                                -> Result<()> {
    //     if let Some(shader) = self.shaders.get_mut(handle) {
    //         shader.uniforms.insert(name.to_string(), *variable);
    //         Ok(())
    //     } else {
    //         bail!(ErrorKind::InvalidHandle);
    //     }
    // }

    /// Free named program object.
    pub unsafe fn delete_shader(&mut self, handle: ShaderHandle) -> Result<()> {
        if let Some(shader) = self.shaders.remove(handle) {
            self.visitor.delete_program(shader.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }
}

struct DataVec<T>
where
    T: Sized,
{
    pub buf: Vec<Option<T>>,
}

impl<T> DataVec<T>
where
    T: Sized,
{
    pub fn new() -> Self {
        DataVec { buf: Vec::new() }
    }

    pub fn get<H>(&self, handle: H) -> Option<&T>
    where
        H: Borrow<Handle>,
    {
        self.buf
            .get(handle.borrow().index() as usize)
            .and_then(|v| v.as_ref())
    }

    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
    where
        H: Borrow<Handle>,
    {
        self.buf
            .get_mut(handle.borrow().index() as usize)
            .and_then(|v| v.as_mut())
    }

    pub fn set<H>(&mut self, handle: H, value: T)
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        while self.buf.len() <= handle.index() as usize {
            self.buf.push(None);
        }

        self.buf[handle.index() as usize] = Some(value);
    }

    pub fn remove<H>(&mut self, handle: H) -> Option<T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        if self.buf.len() <= handle.index() as usize {
            None
        } else {
            let mut value = None;
            ::std::mem::swap(&mut value, &mut self.buf[handle.index() as usize]);
            value
        }
    }
}
