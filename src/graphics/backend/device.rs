use std::str;
use std::cell::Cell;
use std::borrow::Borrow;
use std::collections::HashMap;

use gl;
use gl::types::*;

use utility::Handle;

use super::*;
use super::visitor::*;
use super::super::pipeline::*;
use super::super::resource::*;

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
    attributes: Vec<(GLint, VertexAttributeDesc)>,
    uniforms: HashMap<String, UniformVariable>,
}

#[derive(Debug, Copy, Clone)]
struct GLView {
    viewport: Option<((u16, u16), (u16, u16))>,
    scissor: Option<((u16, u16), (u16, u16))>,
    clear_color: Option<u32>,
    clear_depth: Option<f32>,
    clear_stencil: Option<i32>,
}

pub struct Device {
    visitor: OpenGLVisitor,

    vertex_buffers: DataVec<GLVertexBuffer>,
    index_buffers: DataVec<GLIndexBuffer>,
    pipelines: DataVec<GLPipeline>,
    views: DataVec<GLView>,

    active_view: Cell<Option<ViewHandle>>,
    active_pipeline: Cell<Option<PipelineHandle>>,
}

impl Device {
    pub fn new() -> Self {
        Device {
            visitor: OpenGLVisitor::new(),
            vertex_buffers: DataVec::new(),
            index_buffers: DataVec::new(),
            pipelines: DataVec::new(),
            views: DataVec::new(),
            active_view: Cell::new(None),
            active_pipeline: Cell::new(None),
        }
    }
}

impl Device {
    pub unsafe fn bind_view(&self, view: ViewHandle) -> Result<()> {
        if let Some(v) = self.active_view.get() {
            if v == view {
                return Ok(());
            }
        }

        let vo = self.views.get(view).ok_or(ErrorKind::InvalidHandle)?;
        // TODO set_viewport/ set_scissor
        self.visitor
            .clear(vo.clear_color.map(|v| v.into()),
                   vo.clear_depth,
                   vo.clear_stencil)?;

        self.active_view.set(Some(view));
        Ok(())
    }

    unsafe fn bind_pipeline(&self, pipeline: PipelineHandle) -> Result<&GLPipeline> {
        let pso = self.pipelines.get(pipeline).ok_or(ErrorKind::InvalidHandle)?;

        if let Some(v) = self.active_pipeline.get() {
            if v == pipeline {
                return Ok(&pso);
            }
        }

        self.visitor.bind_program(pso.id)?;
        self.visitor.set_cull_face(pso.state.cull_face)?;
        self.visitor.set_front_face_order(pso.state.front_face_order)?;
        self.visitor.set_depth_test(pso.state.depth_test)?;
        self.visitor.set_depth_write(pso.state.depth_write, pso.state.depth_write_offset)?;
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

    pub unsafe fn draw(&mut self,
                       primitive: Primitive,
                       pipeline: PipelineHandle,
                       vb: VertexBufferHandle,
                       ib: Option<IndexBufferHandle>,
                       from: u32,
                       len: u32,
                       uniforms: &[(&str, UniformVariable)])
                       -> Result<()> {
        let vbo = self.vertex_buffers.get(vb).ok_or(ErrorKind::InvalidHandle)?;
        let pso = self.bind_pipeline(pipeline)?;

        for &(name, variable) in uniforms {
            let location = self.visitor.get_uniform_location(pso.id, &name)?;
            if location != -1 {
                self.visitor.bind_uniform(location, &variable)?;
            }
        }

        self.visitor.bind_attribute_layout(&pso.attributes, &vbo.layout)?;

        if let Some(v) = ib {
            if let Some(ibo) = self.index_buffers.get(v) {
                gl::DrawElements(primitive.into(),
                                 len as GLsizei,
                                 ibo.format.into(),
                                 from as *const u32 as *const ::std::os::raw::c_void);
            } else {
                bail!(ErrorKind::InvalidHandle);
            }
        } else {
            gl::DrawArrays(primitive.into(), from as i32, len as i32);
        }
        check()
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
            id: self.visitor.create_buffer(Resource::Vertex, hint, size, data)?,
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

            self.visitor.update_buffer(vbo.id, Resource::Vertex, offset, data)
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
            id: self.visitor.create_buffer(Resource::Index, hint, size, data)?,
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

            self.visitor.update_buffer(ibo.id, Resource::Index, offset, data)
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

    pub fn create_view(&mut self,
                       handle: ViewHandle,
                       clear_color: Option<u32>,
                       clear_depth: Option<f32>,
                       clear_stencil: Option<i32>)
                       -> Result<()> {
        let view = GLView {
            viewport: None,
            scissor: None,
            clear_color: clear_color,
            clear_depth: clear_depth,
            clear_stencil: clear_stencil,
        };

        self.views.set(handle, view);
        Ok(())
    }

    pub fn update_view_rect(&mut self,
                            handle: ViewHandle,
                            position: (u16, u16),
                            size: (u16, u16))
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
                               size: (u16, u16))
                               -> Result<()> {
        if let Some(view) = self.views.get_mut(handle) {
            view.scissor = Some((position, size));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    pub fn update_view_clear(&mut self,
                             handle: ViewHandle,
                             clear_color: Option<u32>,
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
                                  handle: PipelineHandle,
                                  state: &RenderState,
                                  vs_src: &str,
                                  fs_src: &str,
                                  attributes: (u8, [VertexAttributeDesc; MAX_ATTRIBUTES]))
                                  -> Result<()> {
        let mut pipeline = GLPipeline {
            id: self.visitor.create_program(vs_src, fs_src)?,
            state: *state,
            attributes: Vec::new(),
            uniforms: HashMap::new(),
        };

        for i in 0..attributes.0 {
            let i = i as usize;
            let name: &'static str = attributes.1[i].name.into();
            let location = self.visitor.get_attribute_location(pipeline.id, name)?;
            pipeline.attributes.push((location, attributes.1[i]));
        }

        self.pipelines.set(handle, pipeline);
        check()
    }

    pub fn update_pipeline_state(&mut self,
                                 handle: PipelineHandle,
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
                                   handle: PipelineHandle,
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
    pub unsafe fn delete_pipeline(&mut self, handle: PipelineHandle) -> Result<()> {
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
    buf: Vec<Option<T>>,
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
        self.buf.get(handle.borrow().index() as usize).and_then(|v| v.as_ref())
    }

    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
        where H: Borrow<Handle>
    {
        self.buf.get_mut(handle.borrow().index() as usize).and_then(|v| v.as_mut())
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