use std::str;
use std::cell::Cell;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;

use gl;
use gl::types::*;

use utils::{Handle, Rect, DataBuffer, Color};
use graphics::*;

use super::errors::*;
use super::visitor::*;
use super::frame::{FrameDrawCall, FrameTask};

type ResourceID = GLuint;
type UniformID = GLint;

#[derive(Debug, Clone, Copy)]
struct VertexBufferObject {
    id: ResourceID,
    setup: VertexBufferSetup,
}

#[derive(Debug, Clone, Copy)]
struct IndexBufferObject {
    id: ResourceID,
    setup: IndexBufferSetup,
}

#[derive(Debug)]
struct ShaderObject {
    id: ResourceID,
    render_state: RenderState,
    layout: AttributeLayout,
    uniform_locations: Vec<UniformID>,
    uniforms: HashMap<String, UniformVariable>,
}

#[derive(Debug, Clone)]
struct SurfaceObject {
    setup: SurfaceSetup,
}

#[derive(Debug, Copy, Clone)]
enum GenericTextureSetup {
    Normal(TextureSetup),
    Render(RenderTextureSetup),
}

#[derive(Debug, Copy, Clone)]
struct TextureObject {
    id: ResourceID,
    setup: GenericTextureSetup,
}

#[derive(Debug, Copy, Clone)]
struct RenderBufferObject {
    id: ResourceID,
    setup: RenderBufferSetup,
}

#[derive(Debug, Copy, Clone)]
struct FrameBufferObject {
    id: ResourceID,
}

pub(crate) struct Device {
    visitor: OpenGLVisitor,

    vertex_buffers: DataVec<VertexBufferObject>,
    index_buffers: DataVec<IndexBufferObject>,
    shaders: DataVec<ShaderObject>,
    surfaces: DataVec<SurfaceObject>,
    textures: DataVec<TextureObject>,
    render_buffers: DataVec<RenderBufferObject>,
    framebuffers: DataVec<FrameBufferObject>,

    active_shader: Cell<Option<ShaderHandle>>,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Device {
    pub unsafe fn new() -> Self {
        Device {
            visitor: OpenGLVisitor::new(),
            vertex_buffers: DataVec::new(),
            index_buffers: DataVec::new(),
            shaders: DataVec::new(),
            surfaces: DataVec::new(),
            textures: DataVec::new(),
            render_buffers: DataVec::new(),
            framebuffers: DataVec::new(),
            active_shader: Cell::new(None),
        }
    }
}

impl Device {
    pub unsafe fn run_one_frame(&self) -> Result<()> {
        self.active_shader.set(None);
        self.visitor.bind_framebuffer(0, false)?;
        self.visitor.clear(Color::black(), None, None)?;
        Ok(())
    }

    pub fn flush(&mut self,
                 tasks: &mut [(SurfaceHandle, u64, FrameTask)],
                 buf: &DataBuffer,
                 dimensions: (u32, u32),
                 hidpi: f32)
                 -> Result<()> {
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

        let texture_idx = 0;
        for (i, v) in buf.as_slice(dc.uniforms).iter().enumerate() {
            if let &Some(ptr) = v {
                let variable = buf.as_ref(ptr);
                let location = shader.uniform_locations[i];

                if let &UniformVariable::Texture(handle) = variable {
                    if let Some(texture) = self.textures.get(handle) {
                        let v = UniformVariable::I32(texture_idx);
                        self.visitor.bind_uniform(location, &v)?;
                        self.visitor.bind_texture(texture_idx as u32, texture.id)?;
                    }
                } else {
                    self.visitor.bind_uniform(location, &variable)?;
                }
            }
        }

        // Bind vertex buffer and vertex array object.
        let vbo = self.vertex_buffers
            .get(dc.vb)
            .ok_or(ErrorKind::InvalidHandle)?;

        self.visitor.bind_buffer(gl::ARRAY_BUFFER, vbo.id)?;
        self.visitor
            .bind_attribute_layout(&shader.layout, &vbo.setup.layout)?;

        // Bind index buffer object if available.
        if let Some(v) = dc.ib {
            if let Some(ibo) = self.index_buffers.get(v) {
                self.visitor.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, ibo.id)?;

                let from = dc.from * ibo.setup.format.len() as u32;
                gl::DrawElements(dc.primitive.into(),
                                 dc.len as GLsizei,
                                 ibo.setup.format.into(),
                                 from as *const u32 as *const ::std::os::raw::c_void);
            } else {
                bail!(ErrorKind::InvalidHandle);
            }
        } else {
            gl::DrawArrays(dc.primitive.into(), dc.from as i32, dc.len as i32);
        }

        check()
    }

    unsafe fn rebind_surface(&self,
                             handle: SurfaceHandle,
                             dimensions: (u16, u16),
                             hidpi: f32)
                             -> Result<()> {
        let surface = self.surfaces.get(handle).ok_or(ErrorKind::InvalidHandle)?;

        // Bind frame buffer.
        if let Some(fbo) = surface.setup.framebuffer {
            if let Some(fbo) = self.framebuffers.get(fbo) {
                self.visitor.bind_framebuffer(fbo.id, true)?;
            } else {
                bail!(ErrorKind::InvalidHandle);
            }
        } else {
            self.visitor.bind_framebuffer(0, false)?;
        }

        // Bind the viewport and scissor box.
        let vp = surface.setup.viewport;
        let dimensions = ((dimensions.0 as f32 * hidpi) as u16,
                          (dimensions.1 as f32 * hidpi) as u16);
        self.visitor.set_viewport(vp.0, vp.1.unwrap_or(dimensions))?;

        // Disable scissor.
        self.visitor.set_scissor(Scissor::Disable)?;

        // Clear frame buffer.
        self.visitor
            .clear(surface.setup.clear_color,
                   surface.setup.clear_depth,
                   surface.setup.clear_stencil)?;

        Ok(())
    }

    unsafe fn bind_shader(&self, handle: ShaderHandle) -> Result<&ShaderObject> {
        let shader = self.shaders.get(handle).ok_or(ErrorKind::InvalidHandle)?;

        if let Some(v) = self.active_shader.get() {
            if v == handle {
                return Ok(&shader);
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
            let location = self.visitor.get_uniform_location(shader.id, &name)?;
            if location != -1 {
                self.visitor.bind_uniform(location, &variable)?;
            }
        }

        self.active_shader.set(Some(handle));
        Ok(&shader)
    }
}

impl Device {
    pub unsafe fn create_vertex_buffer(&mut self,
                                       handle: VertexBufferHandle,
                                       setup: VertexBufferSetup,
                                       data: Option<&[u8]>)
                                       -> Result<()> {
        if self.vertex_buffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let vbo = VertexBufferObject {
            id: self.visitor
                .create_buffer(OpenGLBuffer::Vertex, setup.hint, setup.len() as u32, data)?,
            setup: setup,
        };

        self.vertex_buffers.set(handle, vbo);
        check()
    }

    pub unsafe fn update_vertex_buffer(&mut self,
                                       handle: VertexBufferHandle,
                                       offset: usize,
                                       data: &[u8])
                                       -> Result<()> {
        if let Some(vbo) = self.vertex_buffers.get(handle) {
            if vbo.setup.hint == BufferHint::Immutable {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() + offset > vbo.setup.len() {
                bail!(ErrorKind::OutOfBounds);
            }

            self.visitor
                .update_buffer(vbo.id, OpenGLBuffer::Vertex, offset as u32, data)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn delete_vertex_buffer(&mut self, handle: VertexBufferHandle) -> Result<()> {
        if let Some(vbo) = self.vertex_buffers.remove(handle) {
            self.visitor.delete_buffer(vbo.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_index_buffer(&mut self,
                                      handle: IndexBufferHandle,
                                      setup: IndexBufferSetup,
                                      data: Option<&[u8]>)
                                      -> Result<()> {
        if self.index_buffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let ibo = IndexBufferObject {
            id: self.visitor
                .create_buffer(OpenGLBuffer::Index, setup.hint, setup.len() as u32, data)?,
            setup: setup,
        };

        self.index_buffers.set(handle, ibo);
        check()
    }

    pub unsafe fn update_index_buffer(&mut self,
                                      handle: IndexBufferHandle,
                                      offset: usize,
                                      data: &[u8])
                                      -> Result<()> {
        if let Some(ibo) = self.index_buffers.get(handle) {
            if ibo.setup.hint == BufferHint::Immutable {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() + offset > ibo.setup.len() {
                bail!(ErrorKind::OutOfBounds);
            }

            self.visitor
                .update_buffer(ibo.id, OpenGLBuffer::Index, offset as u32, data)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn delete_index_buffer(&mut self, handle: IndexBufferHandle) -> Result<()> {
        if let Some(ibo) = self.index_buffers.remove(handle) {
            self.visitor.delete_buffer(ibo.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_render_buffer(&mut self,
                                       handle: RenderBufferHandle,
                                       setup: RenderBufferSetup)
                                       -> Result<()> {
        let (internal_format, _, _) = setup.format.into();
        let id =
            self.visitor
                .create_render_buffer(internal_format, setup.dimensions.0, setup.dimensions.1)?;

        self.render_buffers
            .set(handle,
                 RenderBufferObject {
                     id: id,
                     setup: setup,
                 });
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

        let fbo = FrameBufferObject { id: self.visitor.create_framebuffer()? };

        self.framebuffers.set(handle, fbo);
        Ok(())
    }

    pub unsafe fn update_framebuffer_with_texture(&mut self,
                                                  handle: FrameBufferHandle,
                                                  texture: TextureHandle,
                                                  slot: u32)
                                                  -> Result<()> {
        let fbo = self.framebuffers
            .get(handle)
            .ok_or(ErrorKind::InvalidHandle)?;

        let texture = self.textures.get(texture).ok_or(ErrorKind::InvalidHandle)?;
        if let GenericTextureSetup::Render(setup) = texture.setup {
            self.visitor.bind_framebuffer(fbo.id, false)?;
            match setup.format {
                RenderTextureFormat::RGB8 |
                RenderTextureFormat::RGBA4 |
                RenderTextureFormat::RGBA8 => {
                    let location = gl::COLOR_ATTACHMENT0 + slot;
                    self.visitor
                        .bind_framebuffer_with_texture(location, texture.id)
                }
                RenderTextureFormat::Depth16 |
                RenderTextureFormat::Depth24 |
                RenderTextureFormat::Depth32 => {
                    self.visitor
                        .bind_framebuffer_with_texture(gl::DEPTH_ATTACHMENT, texture.id)
                }
                RenderTextureFormat::Depth24Stencil8 => {
                    self.visitor
                        .bind_framebuffer_with_texture(gl::DEPTH_STENCIL_ATTACHMENT, texture.id)
                }
            }
        } else {
            bail!("can't attach normal texture to framebuffer.");
        }
    }

    pub unsafe fn update_framebuffer_with_renderbuffer(&mut self,
                                                       handle: FrameBufferHandle,
                                                       buf: RenderBufferHandle,
                                                       slot: u32)
                                                       -> Result<()> {
        let fbo = self.framebuffers
            .get(handle)
            .ok_or(ErrorKind::InvalidHandle)?;
        let buf = self.render_buffers
            .get(buf)
            .ok_or(ErrorKind::InvalidHandle)?;

        self.visitor.bind_framebuffer(fbo.id, false)?;
        match buf.setup.format {
            RenderTextureFormat::RGB8 |
            RenderTextureFormat::RGBA4 |
            RenderTextureFormat::RGBA8 => {
                let location = gl::COLOR_ATTACHMENT0 + slot;
                self.visitor
                    .bind_framebuffer_with_renderbuffer(location, buf.id)
            }
            RenderTextureFormat::Depth16 |
            RenderTextureFormat::Depth24 |
            RenderTextureFormat::Depth32 => {
                self.visitor
                    .bind_framebuffer_with_renderbuffer(gl::DEPTH_ATTACHMENT, buf.id)
            }
            RenderTextureFormat::Depth24Stencil8 => {
                self.visitor
                    .bind_framebuffer_with_renderbuffer(gl::DEPTH_STENCIL_ATTACHMENT, buf.id)
            }
        }
    }

    pub unsafe fn delete_framebuffer(&mut self, handle: FrameBufferHandle) -> Result<()> {
        if let Some(fbo) = self.framebuffers.remove(handle) {
            self.visitor.delete_framebuffer(fbo.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_render_texture(&mut self,
                                        handle: TextureHandle,
                                        setup: RenderTextureSetup)
                                        -> Result<()> {
        let (internal_format, in_format, pixel_type) = setup.format.into();
        let id = self.visitor
            .create_texture(internal_format,
                            in_format,
                            pixel_type,
                            TextureAddress::Repeat,
                            TextureFilter::Linear,
                            false,
                            setup.dimensions.0,
                            setup.dimensions.1,
                            None)?;

        self.textures
            .set(handle,
                 TextureObject {
                     id: id,
                     setup: GenericTextureSetup::Render(setup),
                 });
        Ok(())
    }

    pub unsafe fn create_texture(&mut self,
                                 handle: TextureHandle,
                                 setup: TextureSetup,
                                 data: Option<&[u8]>)
                                 -> Result<()> {
        let (internal_format, in_format, pixel_type) = setup.format.into();
        let id = self.visitor
            .create_texture(internal_format,
                            in_format,
                            pixel_type,
                            setup.address,
                            setup.filter,
                            setup.mipmap,
                            setup.dimensions.0,
                            setup.dimensions.1,
                            data)?;

        self.textures
            .set(handle,
                 TextureObject {
                     id: id,
                     setup: GenericTextureSetup::Normal(setup),
                 });
        Ok(())
    }

    pub unsafe fn update_texture(&mut self,
                                 handle: TextureHandle,
                                 rect: Rect,
                                 data: &[u8])
                                 -> Result<()> {
        if let Some(texture) = self.textures.get(handle) {
            if let GenericTextureSetup::Normal(setup) = texture.setup {
                if data.len() > rect.size() as usize || rect.min.x as u32 >= setup.dimensions.0 ||
                   rect.min.y as u32 >= setup.dimensions.1 ||
                   rect.max.x < 0 || rect.max.y < 0 {
                    bail!(ErrorKind::OutOfBounds);
                }

                let (_, format, tt) = setup.format.into();
                self.visitor
                    .update_texture(texture.id, format, tt, rect, data)?;
                Ok(())
            } else {
                bail!("Can not update render texture.");
            }
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
        if let Some(_) = self.surfaces.remove(handle) {
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

        let mut uniform_locations = Vec::new();
        for name in setup.uniform_variables {
            let location = self.visitor.get_uniform_location(pid, &name)?;
            if location == -1 {
                bail!(format!("failed to locate uniform {:?}", name));
            }

            uniform_locations.push(location);
        }

        self.shaders
            .set(handle,
                 ShaderObject {
                     id: pid,
                     render_state: setup.render_state,
                     layout: setup.layout,
                     uniform_locations: uniform_locations,
                     uniforms: HashMap::new(),
                 });
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
    where T: Sized
{
    pub buf: Vec<Option<T>>,
}

impl<T> DataVec<T>
    where T: Sized
{
    pub fn new() -> Self {
        DataVec { buf: Vec::new() }
    }

    pub fn get<H>(&self, handle: H) -> Option<&T>
        where H: Borrow<Handle>
    {
        self.buf
            .get(handle.borrow().index() as usize)
            .and_then(|v| v.as_ref())
    }

    pub fn set<H>(&mut self, handle: H, value: T)
        where H: Borrow<Handle>
    {
        let handle = handle.borrow();
        while self.buf.len() <= handle.index() as usize {
            self.buf.push(None);
        }

        self.buf[handle.index() as usize] = Some(value);
    }

    pub fn remove<H>(&mut self, handle: H) -> Option<T>
        where H: Borrow<Handle>
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