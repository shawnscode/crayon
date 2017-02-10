use utility::Handle;

use super::state::*;
use super::buffer::*;

/// A `FrameTask` is a self-contained piece of information that
/// is understood by the render backend.
pub enum FrameTask {
    /// Initializes named program object.
    CreateProgram(Handle, DataSlice, DataSlice, Option<DataSlice>),
    /// Free named program object.
    FreeProgram(Handle),

    /// Initialize buffer, named by `handle`, with optional initial data.
    CreateBuffer(Handle, Buffer, BufferHint, u32, Option<DataSlice>),
    /// Update named dynamic `MemoryHint::Dynamic` buffer.
    UpdateBuffer(Handle, DataSlice, u32),
    /// Free named buffer object.
    FreeBuffer(Handle, Handle),

    /// Clear any or all of rendertarget, depth buffer and stencil buffer.
    Clear(Option<[f32; 4]>, Option<f32>, Option<i32>),
    /// Commit render primitives from binding data.
    Draw(usize),
}

pub struct Frame {
    pub buffer: DataBuffer,
    pub tasks: Vec<FrameTask>,
    pub drawcalls: Vec<Drawcall>,
}

// frames: [Arc<Mutex<Frame>>; 2],
// front_frame_index: usize,

impl Frame {
    pub fn allocate(&mut self, data: &[u8]) -> DataSlice {
        self.buffer.add(data)
    }

    pub fn add_task(&mut self, task: FrameTask) {
        self.tasks.push(task)
    }

    pub fn add_drawcall(&mut self, drawcall: Drawcall) {
        self.drawcalls.push(drawcall)
    }
}

pub struct Drawcall {
    
}

/// Where we store all the intermediate bytes.
pub struct DataBuffer(Vec<u8>);

/// The place of some data in the data buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataSlice {
    offset: u32,
    size: u32,
}

impl DataBuffer {
    /// Creates a new empty data buffer.
    pub fn new() -> DataBuffer {
        DataBuffer(Vec::new())
    }

    /// Copy a given raw vector slice into the buffer.
    pub fn add(&mut self, data: &[u8]) -> DataSlice {
        self.0.extend_from_slice(data);
        DataSlice {
            offset: (self.0.len() - data.len()) as u32,
            size: data.len() as u32,
        }
    }

    /// Return a reference to a stored data object.
    pub fn get(&self, slice: DataSlice) -> &[u8] {
        &self.0[slice.offset as usize..(slice.offset + slice.size) as usize]
    }
}