use std::str;
use std::cell::{Cell, RefCell};
use std::borrow::Borrow;
use std::collections::HashMap;

use gl;
use gl::types::*;

use utils::Handle;
use math::Color;

use super::*;
use super::visitor::*;
use super::super::pipeline::*;
use super::super::resource::*;
use super::super::frame::{TaskBuffer, TaskBufferPtr};

type ResourceID = GLuint;

#[derive(Debug, Clone, Copy)]
struct GLVertexBuffer {
    id: ResourceID,
    layout: VertexLayout,
    size: u32,
    hint: ResourceHint,
}

#[derive(Debug, Clone, Copy)]
struct GLIndexBuffer {
    id: ResourceID,
    format: IndexFormat,
    size: u32,
    hint: ResourceHint,
}

#[derive(Debug)]
struct GLPipeline {
    id: ResourceID,
    state: RenderState,
    attributes: AttributeLayout,
    uniforms: HashMap<String, UniformVariable>,
}

#[derive(Debug, Clone)]
struct GLView {
    framebuffer: Option<FrameBufferHandle>,
    viewport: Option<((u16, u16), Option<(u16, u16)>)>,
    scissor: Option<((u16, u16), Option<(u16, u16)>)>,
    priority: u32,
    seq: bool,
    drawcalls: RefCell<Vec<GLDrawcall>>,
    clear_color: Option<Color>,
    clear_depth: Option<f32>,
    clear_stencil: Option<i32>,
}

#[derive(Debug, Copy, Clone)]
enum GLTextureFormat {
    Normal(TextureFormat),
    Render(RenderTextureFormat),
}

#[derive(Debug, Copy, Clone)]
struct GLTexture {
    id: ResourceID,
    address: TextureAddress,
    filter: TextureFilter,
    mipmap: bool,
    width: u32,
    height: u32,
    format: GLTextureFormat,
}

#[derive(Debug, Copy, Clone)]
struct GLRenderTexture {
    id: ResourceID,
    format: GLTextureFormat,
}

#[derive(Debug, Copy, Clone)]
struct GLFrameBuffer {
    id: ResourceID,
}

#[derive(Debug, Copy, Clone)]
struct GLDrawcall {
    priority: u64,
    view: ViewHandle,
    pipeline: PipelineStateHandle,
    uniforms: TaskBufferPtr<[(TaskBufferPtr<str>, UniformVariable)]>,
    textures: TaskBufferPtr<[(TaskBufferPtr<str>, TextureHandle)]>,
    vb: VertexBufferHandle,
    ib: Option<IndexBufferHandle>,
    primitive: Primitive,
    from: u32,
    len: u32,
}

pub struct Device {
    visitor: OpenGLVisitor,

    vertex_buffers: DataVec<GLVertexBuffer>,
    index_buffers: DataVec<GLIndexBuffer>,
    pipelines: DataVec<GLPipeline>,
    views: DataVec<GLView>,
    textures: DataVec<GLTexture>,
    render_textures: DataVec<GLRenderTexture>,
    framebuffers: DataVec<GLFrameBuffer>,

    active_pipeline: Cell<Option<PipelineStateHandle>>,
}

impl Device {
    pub unsafe fn new() -> Self {
        Device {
            visitor: OpenGLVisitor::new(),
            vertex_buffers: DataVec::new(),
            index_buffers: DataVec::new(),
            pipelines: DataVec::new(),
            views: DataVec::new(),
            textures: DataVec::new(),
            render_textures: DataVec::new(),
            framebuffers: DataVec::new(),

            active_pipeline: Cell::new(None),
        }
    }
}

impl Device {
    pub unsafe fn run_one_frame(&self) -> Result<()> {
        for v in self.views.buf.iter() {
            if let Some(vo) = v.as_ref() {
                vo.drawcalls.borrow_mut().clear();
            }
        }

        self.active_pipeline.set(None);
        self.visitor.bind_framebuffer(0, false)?;
        Ok(())
    }

    pub fn submit(&self,
                  priority: u64,
                  view: ViewHandle,
                  pipeline: PipelineStateHandle,
                  textures: TaskBufferPtr<[(TaskBufferPtr<str>, TextureHandle)]>,
                  uniforms: TaskBufferPtr<[(TaskBufferPtr<str>, UniformVariable)]>,
                  vb: VertexBufferHandle,
                  ib: Option<IndexBufferHandle>,
                  primitive: Primitive,
                  from: u32,
                  len: u32)
                  -> Result<()> {
        if let Some(vo) = self.views.get(view) {
            vo.drawcalls
                .borrow_mut()
                .push(GLDrawcall {
                          priority: priority,
                          view: view,
                          pipeline: pipeline,
                          textures: textures,
                          uniforms: uniforms,
                          vb: vb,
                          ib: ib,
                          primitive: primitive,
                          from: from,
                          len: len,
                      });
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn flush(&self, buf: &TaskBuffer, dimensions: (u32, u32)) -> Result<()> {
        // Collects avaiable views.
        let (mut views, mut ordered_views) = (vec![], vec![]);
        for (i, v) in self.views.buf.iter().enumerate() {
            if let Some(vo) = v.as_ref() {
                if vo.priority == 0 {
                    views.push(i);
                } else {
                    ordered_views.push(i);
                }
            }
        }

        // Sort views by user defined priorities.
        ordered_views.sort_by(|lhs, rhs| {
                                  let lv = self.views.buf[*lhs].as_ref().unwrap();
                                  let rv = self.views.buf[*rhs].as_ref().unwrap();
                                  rv.priority.cmp(&lv.priority)
                              });

        let mut uniforms = vec![];
        let mut textures = vec![];
        ordered_views.append(&mut views);

        let dimensions = (dimensions.0 as u16, dimensions.1 as u16);
        for i in ordered_views {
            let vo = self.views.buf[i].as_ref().unwrap();

            // Bind frame buffer and clear it.
            if let Some(fbo) = vo.framebuffer {
                if let Some(fbo) = self.framebuffers.get(fbo) {
                    self.visitor.bind_framebuffer(fbo.id, true)?;
                } else {
                    bail!(ErrorKind::InvalidHandle);
                }
            } else {
                self.visitor.bind_framebuffer(0, false)?;
            }

            // Clear frame buffer.
            self.visitor
                .clear(vo.clear_color, vo.clear_depth, vo.clear_stencil)?;

            // Bind the viewport.
            if let Some(viewport) = vo.viewport {
                self.visitor
                    .set_viewport(viewport.0, viewport.1.unwrap_or(dimensions))?;
            } else {
                self.visitor.set_viewport((0, 0), dimensions)?;
            }

            // Sort bucket drawcalls.
            if !vo.seq {
                vo.drawcalls
                    .borrow_mut()
                    .sort_by(|lhs, rhs| rhs.priority.cmp(&lhs.priority));
            }

            // Submit real OpenGL drawcall in order.
            for dc in vo.drawcalls.borrow().iter() {
                uniforms.clear();
                for &(name, variable) in buf.as_slice(dc.uniforms) {
                    let name = buf.as_str(name);
                    uniforms.push((name, variable));
                }

                textures.clear();
                for &(name, texture) in buf.as_slice(dc.textures) {
                    let name = buf.as_str(name);
                    textures.push((name, texture));
                }

                // Bind program and associated uniforms and textures.
                let pso = self.bind_pipeline(dc.pipeline)?;

                for &(name, variable) in &uniforms {
                    let location = self.visitor.get_uniform_location(pso.id, &name)?;
                    if location == -1 {
                        bail!(format!("failed to locate uniform {}.", &name));
                    }
                    self.visitor.bind_uniform(location, &variable)?;
                }

                for (i, &(name, texture)) in textures.iter().enumerate() {
                    if let Some(to) = self.textures.get(texture) {
                        let location = self.visitor.get_uniform_location(pso.id, &name)?;
                        if location == -1 {
                            bail!(format!("failed to locate texture {}.", &name));
                        }

                        self.visitor
                            .bind_uniform(location, &UniformVariable::I32(i as i32))?;
                        self.visitor.bind_texture(i as u32, to.id)?;
                    } else {
                        bail!(format!("use invalid texture handle {:?} at {}", texture, name));
                    }
                }

                // Bind vertex buffer and vertex array object.
                let vbo = self.vertex_buffers
                    .get(dc.vb)
                    .ok_or(ErrorKind::InvalidHandle)?;
                self.visitor.bind_buffer(gl::ARRAY_BUFFER, vbo.id)?;
                self.visitor
                    .bind_attribute_layout(&pso.attributes, &vbo.layout)?;

                // Bind index buffer object if available.
                if let Some(v) = dc.ib {
                    if let Some(ibo) = self.index_buffers.get(v) {
                        gl::DrawElements(dc.primitive.into(),
                                         dc.len as GLsizei,
                                         ibo.format.into(),
                                         dc.from as *const u32 as *const ::std::os::raw::c_void);
                    } else {
                        bail!(ErrorKind::InvalidHandle);
                    }
                } else {
                    gl::DrawArrays(dc.primitive.into(), dc.from as i32, dc.len as i32);
                }

                check()?;
            }
        }

        Ok(())
    }

    unsafe fn bind_pipeline(&self, pipeline: PipelineStateHandle) -> Result<&GLPipeline> {
        let pso = self.pipelines
            .get(pipeline)
            .ok_or(ErrorKind::InvalidHandle)?;

        if let Some(v) = self.active_pipeline.get() {
            if v == pipeline {
                return Ok(&pso);
            }
        }

        self.visitor.bind_program(pso.id)?;
        self.visitor.set_cull_face(pso.state.cull_face)?;
        self.visitor
            .set_front_face_order(pso.state.front_face_order)?;
        self.visitor.set_depth_test(pso.state.depth_test)?;
        self.visitor
            .set_depth_write(pso.state.depth_write, pso.state.depth_write_offset)?;
        self.visitor.set_color_blend(pso.state.color_blend)?;

        let c = &pso.state.color_write;
        self.visitor.set_color_write(c.0, c.1, c.2, c.3)?;

        for (name, variable) in &pso.uniforms {
            let location = self.visitor.get_uniform_location(pso.id, &name)?;
            if location != -1 {
                self.visitor.bind_uniform(location, &variable)?;
            }
        }

        self.active_pipeline.set(Some(pipeline));
        Ok(&pso)
    }
}

impl Device {
    pub unsafe fn create_vertex_buffer(&mut self,
                                       handle: VertexBufferHandle,
                                       layout: &VertexLayout,
                                       hint: ResourceHint,
                                       size: u32,
                                       data: Option<&[u8]>)
                                       -> Result<()> {
        if self.vertex_buffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let vbo = GLVertexBuffer {
            id: self.visitor
                .create_buffer(Resource::Vertex, hint, size, data)?,
            layout: *layout,
            size: size,
            hint: hint,
        };

        self.vertex_buffers.set(handle, vbo);
        check()
    }

    pub unsafe fn update_vertex_buffer(&mut self,
                                       handle: VertexBufferHandle,
                                       offset: u32,
                                       data: &[u8])
                                       -> Result<()> {
        if let Some(vbo) = self.vertex_buffers.get(handle) {
            if vbo.hint == ResourceHint::Static {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() as u32 + offset > vbo.size {
                bail!(ErrorKind::OutOfBounds);
            }

            self.visitor
                .update_buffer(vbo.id, Resource::Vertex, offset, data)
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
                                      format: IndexFormat,
                                      hint: ResourceHint,
                                      size: u32,
                                      data: Option<&[u8]>)
                                      -> Result<()> {
        if self.index_buffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let ibo = GLIndexBuffer {
            id: self.visitor
                .create_buffer(Resource::Index, hint, size, data)?,
            format: format,
            size: size,
            hint: hint,
        };

        self.index_buffers.set(handle, ibo);
        check()
    }

    pub unsafe fn update_index_buffer(&mut self,
                                      handle: IndexBufferHandle,
                                      offset: u32,
                                      data: &[u8])
                                      -> Result<()> {
        if let Some(ibo) = self.index_buffers.get(handle) {
            if ibo.hint == ResourceHint::Static {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() as u32 + offset > ibo.size {
                bail!(ErrorKind::OutOfBounds);
            }

            self.visitor
                .update_buffer(ibo.id, Resource::Index, offset, data)
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
                                       format: RenderTextureFormat,
                                       width: u32,
                                       height: u32)
                                       -> Result<()> {
        let (internal_format, _, _) = format.into();
        let id = self.visitor
            .create_render_buffer(internal_format, width, height)?;
        self.render_textures
            .set(handle,
                 GLRenderTexture {
                     id: id,
                     format: GLTextureFormat::Render(format),
                 });
        Ok(())
    }

    pub unsafe fn delete_render_buffer(&mut self, handle: RenderBufferHandle) -> Result<()> {
        if let Some(rto) = self.render_textures.remove(handle) {
            self.visitor.delete_render_buffer(rto.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub unsafe fn create_framebuffer(&mut self, handle: FrameBufferHandle) -> Result<()> {
        if self.framebuffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let fbo = GLFrameBuffer { id: self.visitor.create_framebuffer()? };

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
        let tex = self.textures.get(texture).ok_or(ErrorKind::InvalidHandle)?;

        if let GLTextureFormat::Render(format) = tex.format {
            self.visitor.bind_framebuffer(fbo.id, false)?;
            match format {
                RenderTextureFormat::RGB8 |
                RenderTextureFormat::RGBA4 |
                RenderTextureFormat::RGBA8 => {
                    let location = gl::COLOR_ATTACHMENT0 + slot;
                    self.visitor.bind_framebuffer_with_texture(location, tex.id)
                }
                RenderTextureFormat::Depth16 |
                RenderTextureFormat::Depth24 |
                RenderTextureFormat::Depth32 => {
                    self.visitor
                        .bind_framebuffer_with_texture(gl::DEPTH_ATTACHMENT, tex.id)
                }
                RenderTextureFormat::Depth24Stencil8 => {
                    self.visitor
                        .bind_framebuffer_with_texture(gl::DEPTH_STENCIL_ATTACHMENT, tex.id)
                }
            }
        } else {
            bail!("can't attach normal texture to framebuffer.");
        }
    }

    pub unsafe fn update_framebuffer_with_renderbuffer(&mut self,
                                                       handle: FrameBufferHandle,
                                                       texture: RenderBufferHandle,
                                                       slot: u32)
                                                       -> Result<()> {
        let fbo = self.framebuffers
            .get(handle)
            .ok_or(ErrorKind::InvalidHandle)?;
        let tex = self.render_textures
            .get(texture)
            .ok_or(ErrorKind::InvalidHandle)?;

        if let GLTextureFormat::Render(format) = tex.format {
            self.visitor.bind_framebuffer(fbo.id, false)?;
            match format {
                RenderTextureFormat::RGB8 |
                RenderTextureFormat::RGBA4 |
                RenderTextureFormat::RGBA8 => {
                    let location = gl::COLOR_ATTACHMENT0 + slot;
                    self.visitor
                        .bind_framebuffer_with_renderbuffer(location, tex.id)
                }
                RenderTextureFormat::Depth16 |
                RenderTextureFormat::Depth24 |
                RenderTextureFormat::Depth32 => {
                    self.visitor
                        .bind_framebuffer_with_renderbuffer(gl::DEPTH_ATTACHMENT, tex.id)
                }
                RenderTextureFormat::Depth24Stencil8 => {
                    self.visitor
                        .bind_framebuffer_with_renderbuffer(gl::DEPTH_STENCIL_ATTACHMENT, tex.id)
                }
            }
        } else {
            bail!("can't attach normal texture to framebuffer.");
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
                                        format: RenderTextureFormat,
                                        width: u32,
                                        height: u32)
                                        -> Result<()> {
        let (internal_format, in_format, pixel_type) = format.into();
        let id = self.visitor
            .create_texture(internal_format,
                            in_format,
                            pixel_type,
                            TextureAddress::Repeat,
                            TextureFilter::Linear,
                            false,
                            width,
                            height,
                            None)?;

        self.textures
            .set(handle,
                 GLTexture {
                     id: id,
                     address: TextureAddress::Repeat,
                     filter: TextureFilter::Linear,
                     mipmap: false,
                     width: width,
                     height: height,
                     format: GLTextureFormat::Render(format),
                 });
        Ok(())
    }

    pub unsafe fn create_texture(&mut self,
                                 handle: TextureHandle,
                                 format: TextureFormat,
                                 address: TextureAddress,
                                 filter: TextureFilter,
                                 mipmap: bool,
                                 width: u32,
                                 height: u32,
                                 data: &[u8])
                                 -> Result<()> {
        let (internal_format, in_format, pixel_type) = format.into();
        let id = self.visitor
            .create_texture(internal_format,
                            in_format,
                            pixel_type,
                            address,
                            filter,
                            mipmap,
                            width,
                            height,
                            Some(&data))?;

        self.textures
            .set(handle,
                 GLTexture {
                     id: id,
                     address: address,
                     filter: filter,
                     mipmap: mipmap,
                     width: width,
                     height: height,
                     format: GLTextureFormat::Normal(format),
                 });
        Ok(())
    }

    pub unsafe fn update_texture_parameters(&mut self,
                                            handle: TextureHandle,
                                            address: TextureAddress,
                                            filter: TextureFilter)
                                            -> Result<()> {
        if let Some(texture) = self.textures.get_mut(handle) {
            self.visitor.bind_texture(0, texture.id)?;
            self.visitor
                .update_texture_parameters(address, filter, texture.mipmap)?;
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

    pub fn create_view(&mut self,
                       handle: ViewHandle,
                       framebuffer: Option<FrameBufferHandle>)
                       -> Result<()> {
        let view = GLView {
            viewport: None,
            scissor: None,
            framebuffer: framebuffer,
            seq: false,
            priority: 0,
            drawcalls: RefCell::new(Vec::new()),
            clear_color: None,
            clear_depth: None,
            clear_stencil: None,
        };

        self.views.set(handle, view);
        Ok(())
    }

    pub fn update_view_rect(&mut self,
                            handle: ViewHandle,
                            position: (u16, u16),
                            size: Option<(u16, u16)>)
                            -> Result<()> {
        if let Some(view) = self.views.get_mut(handle) {
            view.viewport = Some((position, size));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn update_view_scissor(&mut self,
                               handle: ViewHandle,
                               position: (u16, u16),
                               size: Option<(u16, u16)>)
                               -> Result<()> {
        if let Some(view) = self.views.get_mut(handle) {
            view.scissor = Some((position, size));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn update_view_order(&mut self, handle: ViewHandle, priority: u32) -> Result<()> {
        if let Some(view) = self.views.get_mut(handle) {
            view.priority = priority;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn update_view_sequential_mode(&mut self, handle: ViewHandle, seq: bool) -> Result<()> {
        if let Some(view) = self.views.get_mut(handle) {
            view.seq = seq;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn update_view_framebuffer(&mut self,
                                   handle: ViewHandle,
                                   framebuffer: Option<FrameBufferHandle>)
                                   -> Result<()> {
        if let Some(view) = self.views.get_mut(handle) {
            view.framebuffer = framebuffer;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn update_view_clear(&mut self,
                             handle: ViewHandle,
                             clear_color: Option<Color>,
                             clear_depth: Option<f32>,
                             clear_stencil: Option<i32>)
                             -> Result<()> {
        if let Some(view) = self.views.get_mut(handle) {
            view.clear_color = clear_color;
            view.clear_depth = clear_depth;
            view.clear_stencil = clear_stencil;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn delete_view(&mut self, handle: ViewHandle) -> Result<()> {
        if let Some(_) = self.views.remove(handle) {
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Initializes named program object. A program object is an object to
    /// which shader objects can be attached. Vertex and fragment shader
    /// are minimal requirement to build a proper program.
    pub unsafe fn create_pipeline(&mut self,
                                  handle: PipelineStateHandle,
                                  state: &RenderState,
                                  vs_src: &str,
                                  fs_src: &str,
                                  attributes: &AttributeLayout)
                                  -> Result<()> {

        let pid = self.visitor.create_program(vs_src, fs_src)?;

        for (name, _) in attributes.iter() {
            let name: &'static str = name.into();
            let location = self.visitor.get_attribute_location(pid, name)?;
            if location == -1 {
                bail!(format!("failed to locate attribute {:?}", name));
            }
        }

        self.pipelines
            .set(handle,
                 GLPipeline {
                     id: pid,
                     state: *state,
                     attributes: *attributes,
                     uniforms: HashMap::new(),
                 });
        check()
    }

    pub fn update_pipeline_state(&mut self,
                                 handle: PipelineStateHandle,
                                 state: &RenderState)
                                 -> Result<()> {
        if let Some(pso) = self.pipelines.get_mut(handle) {
            pso.state = *state;
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn update_pipeline_uniform(&mut self,
                                   handle: PipelineStateHandle,
                                   name: &str,
                                   variable: &UniformVariable)
                                   -> Result<()> {
        if let Some(pso) = self.pipelines.get_mut(handle) {
            pso.uniforms.insert(name.to_string(), *variable);
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Free named program object.
    pub unsafe fn delete_pipeline(&mut self, handle: PipelineStateHandle) -> Result<()> {
        if let Some(pso) = self.pipelines.remove(handle) {
            self.visitor.delete_program(pso.id)
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

    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
        where H: Borrow<Handle>
    {
        self.buf
            .get_mut(handle.borrow().index() as usize)
            .and_then(|v| v.as_mut())
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