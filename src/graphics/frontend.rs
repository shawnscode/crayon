use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

use utility::{HandleSet, HandleSetWith, Handle};
use super::buffer::VertexLayout;
use super::state::State;
use super::frame;

pub struct Frontend {
    views: HandleSetWith<View>,
    states: HandleSetWith<State>,

    frames: [Arc<Mutex<frame::Frame>>; 2],
    front_frame_index: usize,
}

impl Frontend {
    /// Advance to next frame. This call just swaps internal buffers, kicks
    /// render thread, and returns.
    pub fn run_one_frame() {}

    #[inline]
    pub fn create_state(&mut self, state: State) -> Handle {
        self.states.create(state)
    }

    #[inline]
    pub fn state(&mut self, handle: Handle) -> Option<&State> {
        self.states.get(handle)
    }

    #[inline]
    pub fn state_mut(&mut self, handle: Handle) -> Option<&mut State> {
        self.states.get_mut(handle)
    }

    #[inline]
    pub fn create_view(&mut self, view: View) -> Handle {
        self.views.create(view)
    }

    #[inline]
    pub fn view(&mut self, handle: Handle) -> Option<&View> {
        self.views.get(handle)
    }

    #[inline]
    pub fn view_mut(&mut self, handle: Handle) -> Option<&mut View> {
        self.views.get_mut(handle)
    }

    // pub fn submit(drawcall: Drawcall, )

    // pub fn create_vertex_buffer(&mut self,
    //                             layout: &VertexLayout,
    //                             buffer: BufferHint,
    //                             data: Option<&[u8]>)
    //                             -> Handle {
    //     let handle = self.buffers.create();
    //     handle
    // }

    // pub fn update_vertex_buffer(&mut self, handle: Handle, offset: u32, data: Option<&[u8]>) {
    //     if self.buffers.is_alive(handle) {

    //     }
    // }

    // pub fn free_vertex_buffer(&mut self, handle: Handle) {
    //     self.buffers.free(handle);
    // }

    // pub fn create_
}

/// View is primary sorting mechanism in lemon3d. View represent bucket of drawcalls,
/// while drawcalls are sorted by internal state if View is not in sequential mode.
///
/// In case where order has to be preserved, for example in rendering GUI, view can
/// be set to be in sequential order, its less efficient, because it doesn't allow state
/// change optimization, and should be avoided when possible.
#[derive(Debug, Default, Clone, Copy)]
pub struct View {
    priority: u8,
    sequential: bool,
    viewport: Option<((u32, u32), (u32, u32))>,
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    clear_stencil: Option<i32>,
}

impl View {
    /// Create a new and empty View.
    pub fn new() -> View {
        Default::default()
    }

    /// Post submit view order.
    pub fn set_order(&mut self, priority: u8) {
        self.priority = priority;
    }

    /// Set viewport rectangle. Draw primitive outside view will be clipped.
    pub fn set_viewport(&mut self, position: (u32, u32), size: (u32, u32)) {
        self.viewport = Some((position, size));
    }

    /// Set clear flags.
    pub fn set_clear(&mut self,
                     color: Option<[f32; 4]>,
                     depth: Option<f32>,
                     stencil: Option<i32>) {
        self.clear_color = color;
        self.clear_depth = depth;
        self.clear_stencil = stencil;
    }

    /// Set view into sequential mode. Draw calls will be sorted in the same order
    /// in which submit calls were called.
    pub fn set_sequential_mode(&mut self, sequential: bool) {
        self.sequential = sequential;
    }

    // fn bind(backend: Device) {
    //     backend.
    // }
}

// pub struct Capacilities {
//     pub max_draw_calls: u32,
//     pub max_blits: u32,
//     pub max_texture_size: u32,
//     pub max_views: u32,
//     pub max_buffers: u32,
//     /// ... etc.
// }