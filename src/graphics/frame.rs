use std::str;
use utility::Handle;
use super::buffer::*;

/// A `Task` is a self-contained piece of information that is understood by
/// the render backend. And for the sack of correctness, `Task` will be executed in order.
/// Backend will execute `Creation` first, and then `Frame`, and then `Destruction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationTask {
    /// Initializes named `PipelineStateObject`.
    CreatePipelineState {
        handle: Handle,
        vs: DataSlice,
        fs: DataSlice,
        state: DataSlice,
    },

    /// Initializes named `ViewObject`.
    CreateView { handle: Handle, view: DataSlice },

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
    FreePipelineState(Handle),
    /// Free named `ViewObject`.
    FreeView(Handle),
    /// Free named buffer object.
    FreeVertexBuffer(Handle),
}

#[derive(Debug, Default)]
pub struct Frame {
    pub buffer: DataBuffer,
    pub creation_tasks: Vec<CreationTask>,
    pub frame_tasks: Vec<FrameTask>,
    pub destruction_tasks: Vec<DestructionTask>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            buffer: DataBuffer::new(),
            creation_tasks: Vec::new(),
            frame_tasks: Vec::new(),
            destruction_tasks: Vec::new(),
        }
    }

    pub fn allocate(&mut self, data: &[u8]) -> DataSlice {
        self.buffer.add(data)
    }

    pub fn clear(&mut self) {
        self.buffer.0.clear();
        self.creation_tasks.clear();
        self.frame_tasks.clear();
        self.destruction_tasks.clear();
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
    pub fn add(&mut self, data: &[u8]) -> DataSlice {
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