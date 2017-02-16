use std::borrow::Borrow;

#[macro_use]
use utility::Handle;
use utility::{HandleSet, HandleObjectSet};
use utility::hash::*;
use utility::memory::*;

use super::color::*;
use super::state::*;
use super::frame::*;
use super::buffer::*;
use super::shader::*;

pub struct Graphics {
    views: HandleObjectSet<ViewStateObject>,
    pipelines: HandleObjectSet<PipelineStateObject>,
    vertex_buffers: HandleObjectSet<VertexBufferObject>,
    uniforms: DataBuffer,
    frame: Frame, // frontend: Frontend,
}


/// View 1. When: In What Order; 2. Where: Viewport/ Scissor/ Framebuffer.
/// Pipeline 1. How: RenderState/ Shader/ UniformLayout/ AttributeLayout/ Shared Uniforms.
/// Drawcall 1. What(Who): VertexBuffer/ IndexBuffer/ PerDraw Uniforms.
impl_handle!(ViewHandle);
impl_handle!(PipelineHandle);

impl_handle!(FrameBufferHandle);
impl_handle!(VertexBufferHandle);
impl_handle!(IndexBufferHandle);

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            vertex_buffers: HandleObjectSet::new(),
            pipelines: HandleObjectSet::new(),
            views: HandleObjectSet::new(),

            frame: Frame::new(),
            uniforms: DataBuffer::new(),
        }
    }

    /// Advance to next frame. This call just swaps internal buffers, kicks
    /// render thread, and returns.
    pub fn run_one_frame(&mut self) {
        unsafe {
            let mut frame = Frame::new();
            ::std::mem::swap(&mut frame, &mut self.frame);
        }
    }
}

/// View is primary sorting mechanism in lemon3d. View represent bucket of drawcalls,
/// while drawcalls are sorted by internal state if View is not in sequential mode.
///
/// In case where order has to be preserved, for example in rendering GUI, view can
/// be set to be in sequential order, its less efficient, because it doesn't allow state
/// change optimization, and should be avoided when possible.
#[derive(Debug, Default, Copy, Clone)]
pub struct ViewStateObject {
    pub viewport: Option<((u32, u32), (u32, u32))>,
    pub scissor: Option<((u32, u32), (u32, u32))>,
    pub clear_color: Option<u32>,
    pub clear_depth: Option<f32>,
    pub clear_stencil: Option<i32>,
    pub framebuffer: Option<FrameBufferHandle>,
    pub shared_uniforms: Vec<(u64, DataSlice)>,
}

impl Graphics {
    /// Allocate a handle and space for view object.
    pub fn create_view(&mut self) -> ViewHandle {
        self.views.create(ViewStateObject::default()).into()
    }

    /// Set view's viewport. Draw primitive outsize viewport will be clipped.
    pub fn set_view_rect(&mut self, handle: ViewHandle, position: (u32, u32), size: (u32, u32)) {
        if let Some(vso) = self.views.get_mut(handle) {
            vso.viewport = Some((position, size));
        }
    }

    /// Set view's frame buffer. Passing `None` as frame buffer will draw
    /// primitives from this view into default back buffer.
    pub fn set_view_framebuffer(&mut self,
                                handle: ViewHandle,
                                framebuffer: Option<FrameBufferHandle>) {
        if let Some(vso) = self.views.get_mut(handle) {
            vso.framebuffer = framebuffer;
        }
    }

    /// Set view's clear flag.
    pub fn set_view_clear(&mut self,
                          handle: ViewHandle,
                          color: Option<Color>,
                          depth: Option<f32>,
                          stencil: Option<i32>) {
        if let Some(vso) = self.views.get_mut(handle) {
            vso.clear_color = color.map(|c| c.into());
            vso.clear_depth = depth;
            vso.clear_stencil = stencil;
        }
    }

    /// Set view's shared uniforms, like view and projection matrices. All draw primitives
    /// commit with this view will use these uniforms.
    pub fn set_view_shared_uniforms<T, F>(&mut self, handle: ViewHandle, uniforms: &Vec<(T, F)>)
        where T: Borrow<str>,
              F: Borrow<[f32]>
    {
        if let Some(vso) = self.views.get_mut(handle) {
            let name = hash(&name.borrow());
            let slice = self.uniforms.allocate(&cast_slice(uniform.borrow()));
            vso.shared_uniforms.push((name, slice));
        }
    }

    /// Free named view state object.
    pub fn free_view(&mut self, handle: ViewHandle) {
        self.views.free(handle);
    }
}

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct PipelineAttribute(VertexFormat, u8);

/// PipelineStateObject encapsulates all the descriptions about how to render a vertex into
/// framebuffer.
#[derive(Debug, Clone)]
pub struct PipelineStateObject {
    pub state: RenderState,
    pub attributes: HashMap<String, PipelineAttribute>,
    pub uniforms: HashSet<String>,
}

impl Graphics {
    /// Creates pipeline object with programable shader and its descriptions.
    pub fn create_pipeline<T>(&mut self, vs: T, fs: T) -> PipelineHandle
        where T: Borrow<str>
    {
        let pso = PipelineStateObject {
            state: RenderState::default(),
            attributes: HashMap::new(),
            uniforms: HashSet::new(),
            shared_uniforms: Vec::new(),
        };

        let handle = self.pipelines.create(pso);
        let vs = self.frame.allocate(vs.borrow().as_bytes());
        let fs = self.frame.allocate(fs.borrow().as_bytes());
        self.frame.creation_tasks.push(CreationTask::CreateProgram(handle, vs, fs));

        handle.into()
    }

    /// Set pipeline's render state.
    pub fn set_pipeline_state(&mut self, handle: PipelineHandle, state: &RenderState) {
        if let Some(pso) = self.pipelines.get_mut(handle) {
            pso.state = *state;
        }
    }

    /// Free named pipeline state object.
    pub fn free_pipeline(&mut self, handle: PipelineHandle) {
        if self.pipelines.is_alive(handle) {
            self.frame.destruction_tasks.push(DestructionTask::FreeProgram(handle.0));
        }
    }
}

struct VertexBufferObject {
    pub layout: VertexLayout,
    pub size: u32,
    pub hint: BufferHint,
    pub handle: Handle,
}

impl Graphics {
    pub fn create_vertex_buffer(&mut self,
                                layout: &VertexLayout,
                                size: u32,
                                hint: BufferHint,
                                data: Option<&[u8]>) {
        // let handle = self.vertex_buffers.create();
        // let slice = data.map(|v| self.frame.allocate(v));
        // self.frame
        //     .creation_tasks
        //     .push(CreationTask::CreateVertexBuffer {
        //         handle: handle,
        //         layout: *layout,
        //         size: size,
        //         hint: hint,
        //         data: slice,
        //     });
        // handle
    }

    pub fn update_vertex_buffer(&mut self, handle: Handle, offset: u32, data: &[u8]) {
        // if self.vertex_buffers.is_alive(handle) {
        //     let slice = self.frame.allocate(data);
        //     self.frame.creation_tasks.push(CreationTask::UpdateVertexBuffer {
        //         handle: handle,
        //         offset: offset,
        //         data: slice,
        //     });
        // }
    }

    pub fn free_vertex_buffer(&mut self, handle: Handle) {
        // if self.vertex_buffers.is_alive(handle) {
        //     self.frame.destruction_tasks.push(DestructionTask::FreeVertexBuffer(handle));
        //     self.vertex_buffers.free(handle);
        // }
    }
}
