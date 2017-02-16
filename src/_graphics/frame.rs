use std::str;
use math;
use utility::Handle;

use super::state::*;
use super::shader::*;
use super::buffer::*;
use super::graphics::*;

/// A `Task` is a self-contained piece of information that is understood by
/// the render backend. And for the sack of correctness, `Task` will be executed in order.
/// Backend will execute `Creation` first, and then `Frame`, and then `Destruction`.
#[derive(Debug, Clone, Copy)]
pub enum CreationTask {
    CreateProgram(Handle, DataSlice, DataSlice),

    /// Initializes buffer, named by `Handle`, with optional initial data.
    CreateVertexBuffer {
        handle: Handle,
        layout: VertexLayout,
        size: u32,
        hint: BufferHint,
        data: Option<DataSlice>,
    },

    /// Update named dynamic `MemoryHint::Dynamic` buffer.
    UpdateVertexBuffer {
        handle: Handle,
        offset: u32,
        data: DataSlice,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameTask {
    /// Commit render primitives from binding data.
    Draw(Handle, Handle, Drawcall),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructionTask {
    /// Free named `PipelineStateObject`.
    FreeProgram(Handle),
    /// Free named buffer object.
    FreeVertexBuffer(Handle),
}

#[derive(Debug, Default)]
pub struct Frame {
    pub buffer: DataBuffer,
    pub creation_tasks: Vec<CreationTask>,
    pub destruction_tasks: Vec<DestructionTask>,

    pub views: Vec<Option<ViewStateObject>>,
    pub pipelines: Vec<Option<RenderState>>,
    pub drawcalls: Vec<(u32, u32, Drawcall)>,
}

impl Frame {
    pub fn new() -> Frame {
        Default::default()
    }

    #[inline]
    pub fn allocate(&mut self, data: &[u8]) -> DataSlice {
        self.buffer.allocate(data)
    }

    #[inline]
    pub fn submit_drawcall(&mut self, vso: u32, pso: u32, drawcall: &Drawcall) {
        self.drawcalls.push((vso, pso, drawcall));
    }

    #[inline]
    pub fn submit_vso(&mut self, idx: u32, vso: &ViewStateObject) {
        if idx as usize >= self.views.len() {
            self.views.resize(idx as usize + 1, None);
        }

        self.views[idx as usize] = Some(vso);
    }

    #[inline]
    pub fn submit_pso(&mut self, idx: u32, pso: &PipelineStateObject) {
        if idx as usize >= self.views.len() {
            self.pipelines.resize(idx as usize + 1, None);
        }

        self.pipelines[ids as usize] = Some(pso.state);
    }

    pub fn clear(&mut self) {
        self.buffer.0.clear();
        self.creation_tasks.clear();
        self.destruction_tasks.clear();
        self.drawcalls.clear();
        self.views.clear();
        self.pipelines.clear();
    }
}

/// All the data we need for rendering primitive.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Drawcall {
    pub vertices: Handle,
    pub indices: Handle,
    pub offset: u32,
    pub size: u32,
}

/// Where we store all the intermediate bytes.
#[derive(Debug, Default)]
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
    pub fn allocate(&mut self, data: &[u8]) -> DataSlice {
        self.0.extend_from_slice(data);
        DataSlice {
            offset: (self.0.len() - data.len()) as u32,
            size: data.len() as u32,
        }
    }

    /// Return a reference to a stored data object.
    #[inline]
    pub fn get(&self, slice: DataSlice) -> &[u8] {
        &self.0[slice.offset as usize..(slice.offset + slice.size) as usize]
    }

    #[inline]
    pub fn get_str(&self, slice: DataSlice) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.get(slice))
    }
}