use bincode;

use utility::Handle;
use super::frame::*;
use super::state::*;
use super::buffer::*;
use super::backend::*;

/// PipelineStateObject encapsulates all the descriptions about how to render a vertex into
/// framebuffer.
#[derive(Debug, Clone, Copy)]
struct PipelineStateObject {
    pub state: RenderState,
    pub handle: Handle,
}

/// View is primary sorting mechanism in lemon3d. View represent bucket of drawcalls,
/// while drawcalls are sorted by internal state if View is not in sequential mode.
///
/// In case where order has to be preserved, for example in rendering GUI, view can
/// be set to be in sequential order, its less efficient, because it doesn't allow state
/// change optimization, and should be avoided when possible.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ViewObject {
    pub priority: u8,
    pub sequential: bool,
    pub viewport: Option<((u32, u32), (u32, u32))>,
    pub clear_color: Option<[f32; 4]>,
    pub clear_depth: Option<f32>,
    pub clear_stencil: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub struct VertexBufferObject {
    pub layout: VertexLayout,
    pub size: u32,
    pub hint: BufferHint,
    pub handle: Handle,
}

pub struct Frontend {
    views: HandledObjects<ViewObject>,
    pipelines: HandledObjects<PipelineStateObject>,
    vertex_buffers: HandledObjects<VertexBufferObject>,
    device: device::Device,
}

impl Frontend {
    pub fn new() -> Frontend {
        Frontend {
            views: HandledObjects::new(),
            pipelines: HandledObjects::new(),
            vertex_buffers: HandledObjects::new(),
            device: device::Device::new(),
        }
    }

    pub unsafe fn run_one_frame(&mut self, frame: Frame) {
        let creation_tasks = &frame.creation_tasks;
        let destruction_tasks = &frame.destruction_tasks;
        let buffer = &frame.buffer;

        for v in creation_tasks.into_iter() {
            self.run_creation_task(v, buffer);
        }

        let mut buckets = vec![Vec::new(); self.views.0.len()];
        for task in frame.frame_tasks.iter() {
            match *task {
                FrameTask::Draw(view, pso, drawcall) => {
                    buckets[view.index() as usize].push((pso, drawcall));
                }
            }
        }

        for (i, b) in buckets.iter().enumerate() {
            if let Some(view) = self.views.0[i] {
                if let Some((position, size)) = view.viewport {
                    self.device.set_viewport(position, size);
                }
                self.device.clear(view.clear_color, view.clear_depth, view.clear_stencil);
            }

            // for v in b {
            //     if let Some(pso) = self.pipelines.get(v.0) {
            //         self.set_render_state(&pso.state);
            //     }
            // }
        }

        for task in destruction_tasks {
            self.run_destruction_task(task);
        }
    }

    unsafe fn run_creation_task(&mut self, task: &CreationTask, buffer: &DataBuffer) {
        match *task {
            CreationTask::CreatePipelineState { handle, vs, fs, state } => {
                let pso = PipelineStateObject {
                    state: bincode::deserialize::<RenderState>(buffer.get(state)).unwrap(),
                    handle: self.device.create_program(buffer.get_str(vs).unwrap(),
                                                       buffer.get_str(fs).unwrap(),
                                                       None),
                };
                self.pipelines.insert(handle, pso);
            }

            CreationTask::CreateView { handle, view } => {
                let vo = bincode::deserialize::<ViewObject>(buffer.get(view)).unwrap();
                self.views.insert(handle, vo);
            }

            CreationTask::CreateVertexBuffer { handle, layout, hint, size, data } => {
                let data = data.map(|w| buffer.get(w));
                let vbo = VertexBufferObject {
                    layout: layout,
                    size: size,
                    hint: hint,
                    handle: self.device.create_buffer(Buffer::Vertex, hint, size, data),
                };

                self.vertex_buffers.insert(handle, vbo);
            }

            CreationTask::UpdateVertexBuffer { handle, offset, data } => {
                if let Some(vbo) = self.vertex_buffers.get(handle) {
                    let data = buffer.get(data);
                    self.device.update_buffer(handle, offset, data);
                }
            }
        }
    }

    unsafe fn run_destruction_task(&mut self, task: &DestructionTask) {
        match *task {
            DestructionTask::FreePipelineState(psh) => {
                if let Some(&PipelineStateObject { handle, .. }) = self.pipelines.get(psh) {
                    self.device.free_program(handle);
                    self.pipelines.remove(psh);
                }
            }

            DestructionTask::FreeView(vh) => {
                self.views.remove(vh);
            }

            DestructionTask::FreeVertexBuffer(vbh) => {
                if let Some(&VertexBufferObject { handle, .. }) = self.vertex_buffers.get(vbh) {
                    self.device.free_buffer(handle);
                    self.vertex_buffers.remove(vbh);
                }
            }
        }
    }

    unsafe fn set_render_state(&mut self, s: &RenderState) {
        let c = &s.color_write;
        self.device.set_face_cull(s.cull_face);
        self.device.set_front_face(s.front_face_order);
        self.device.set_depth_test(s.depth_test);
        self.device.set_depth_write(s.depth_write, s.depth_write_offset);
        self.device.set_color_blend(s.color_blend);
        self.device.set_color_write(c.0, c.1, c.2, c.3);
    }
}

impl Default for ViewObject {
    fn default() -> Self {
        ViewObject {
            priority: 0,
            sequential: false,
            viewport: None,
            clear_color: None,
            clear_depth: None,
            clear_stencil: None,
        }
    }
}

impl ViewObject {
    /// Create a new and empty `ViewObject`.
    pub fn new() -> ViewObject {
        Default::default()
    }

    /// Post submit view order.
    #[inline]
    pub fn with_order(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set viewport rectangle. Draw primitive outside view will be clipped.
    #[inline]
    pub fn with_viewport(mut self, position: (u32, u32), size: (u32, u32)) -> Self {
        self.viewport = Some((position, size));
        self
    }

    /// Set clear flags.
    #[inline]
    pub fn with_clear(mut self,
                      color: Option<[f32; 4]>,
                      depth: Option<f32>,
                      stencil: Option<i32>)
                      -> Self {
        self.clear_color = color;
        self.clear_depth = depth;
        self.clear_stencil = stencil;
        self
    }

    /// Set view into sequential mode. Draw calls will be sorted in the same order
    /// in which submit calls were called.
    pub fn with_sequential_mode(mut self, sequential: bool) -> Self {
        self.sequential = sequential;
        self
    }
}

struct HandledObjects<T: Sized + Clone>(Vec<Option<T>>);

impl<T: Sized + Clone> HandledObjects<T> {
    pub fn new() -> Self {
        HandledObjects(Vec::new())
    }

    pub fn insert(&mut self, handle: Handle, value: T) {
        if self.0.len() <= handle.index() as usize {
            self.0.resize(handle.index() as usize + 1, None);
        }

        self.0[handle.index() as usize] = Some(value)
    }

    pub fn remove(&mut self, handle: Handle) {
        if self.0.len() > handle.index() as usize {
            self.0[handle.index() as usize] = None
        }
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        match self.0.get(handle.index() as usize) {
            None => None,
            Some(v) => v.as_ref(),
        }
    }
}
