//! #### Graphics Subsystem
//! Graphics is the most fundamental subsystem of [Lemon3d](http://github.com/kayak233/lemon3d).
//! It was degisned to provide a set of stateless and high-performance
//! graphics APIs based on OpenGL.

pub mod state;
pub mod buffer;
pub mod backend;
pub mod frontend;

pub use self::frontend::ViewObject;

mod frame;
use bincode;
use std::borrow::Borrow;

use utility::{Handle, HandleSet};
use self::state::*;
use self::frame::*;
use self::buffer::*;
use self::frontend::*;

const MAX_VERTEX_ATTRIBUTES: usize = 8;


pub struct Graphics {
    pipelines: HandleSet,
    views: HandleSet,
    vertex_buffers: HandleSet,
    frame: Frame,
    frontend: Frontend,
}

// pub trait Vertex {
//     pub fn
// }

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            pipelines: HandleSet::new(),
            views: HandleSet::new(),
            vertex_buffers: HandleSet::new(),

            frame: Frame::new(),
            frontend: Frontend::new(),
        }
    }

    /// Advance to next frame. This call just swaps internal buffers, kicks
    /// render thread, and returns.
    pub fn run_one_frame(&mut self) {
        unsafe {
            let mut frame = Frame::new();
            ::std::mem::swap(&mut frame, &mut self.frame);
            self.frontend.run_one_frame(frame);
        }
    }

    pub fn create_pipeline_state<T>(&mut self, state: &RenderState, vs: T, fs: T) -> Handle
        where T: Borrow<str>
    {
        let handle = self.pipelines.create();
        let encoded = bincode::serialize(state, bincode::SizeLimit::Infinite).unwrap();
        let state = self.frame.allocate(&encoded);
        let vs = self.frame.allocate(vs.borrow().as_bytes());
        let fs = self.frame.allocate(fs.borrow().as_bytes());
        self.frame
            .creation_tasks
            .push(CreationTask::CreatePipelineState {
                handle: handle,
                state: state,
                vs: vs,
                fs: fs,
            });
        handle
    }

    pub fn free_pipeline_state(&mut self, handle: Handle) {
        if self.pipelines.is_alive(handle) {
            self.frame.destruction_tasks.push(DestructionTask::FreePipelineState(handle));
            self.pipelines.free(handle);
        }
    }

    pub fn create_view(&mut self, view: &ViewObject) -> Handle {
        let handle = self.views.create();
        let encoded = bincode::serialize(view, bincode::SizeLimit::Infinite).unwrap();
        let view = self.frame.allocate(&encoded);
        self.frame.creation_tasks.push(CreationTask::CreateView {
            handle: handle,
            view: view,
        });
        handle
    }

    // pub fn set_view_transform(&mut self)

    pub fn free_view(&mut self, handle: Handle) {
        if self.views.is_alive(handle) {
            self.frame.destruction_tasks.push(DestructionTask::FreeView(handle));
            self.views.free(handle);
        }
    }

    pub fn create_vertex_buffer(&mut self,
                                layout: &VertexLayout,
                                size: u32,
                                hint: BufferHint,
                                data: Option<&[u8]>)
                                -> Handle {
        let handle = self.vertex_buffers.create();
        let slice = data.map(|v| self.frame.allocate(v));
        self.frame
            .creation_tasks
            .push(CreationTask::CreateVertexBuffer {
                handle: handle,
                layout: *layout,
                size: size,
                hint: hint,
                data: slice,
            });
        handle
    }

    pub fn update_vertex_buffer(&mut self, handle: Handle, offset: u32, data: &[u8]) {
        if self.vertex_buffers.is_alive(handle) {
            let slice = self.frame.allocate(data);
            self.frame.creation_tasks.push(CreationTask::UpdateVertexBuffer {
                handle: handle,
                offset: offset,
                data: slice,
            });
        }
    }

    pub fn free_vertex_buffer(&mut self, handle: Handle) {
        if self.vertex_buffers.is_alive(handle) {
            self.frame.destruction_tasks.push(DestructionTask::FreeVertexBuffer(handle));
            self.vertex_buffers.free(handle);
        }
    }
}
