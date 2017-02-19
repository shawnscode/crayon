use std::marker::PhantomData;
use std::borrow::Borrow;
use std::str;
use std::slice;
use std::mem;

use super::*;
use super::resource::{ResourceHint, VertexLayout, VertexAttributeDesc, MAX_ATTRIBUTES};
use super::pipeline::UniformVariable;

#[derive(Debug, Clone, Copy)]
pub enum PreFrameTask {
    CreateView(ViewHandle),
    UpdateViewRect(ViewHandle, (u16, u16), (u16, u16)),
    UpdateViewScissor(ViewHandle, (u16, u16), (u16, u16)),
    UpdateViewClear(ViewHandle, Option<u32>, Option<f32>, Option<i32>),

    CreatePipeline(PipelineHandle, TaskBufferPtr<PipelineDescriptor>),
    UpdatePipelineState(PipelineHandle, TaskBufferPtr<RenderState>),
    UpdatePipelineUniform(PipelineHandle, TaskBufferPtr<str>, TaskBufferPtr<UniformVariable>),

    CreateVertexBuffer(VertexBufferHandle, TaskBufferPtr<VertexBufferDescriptor>),
    UpdateVertexBuffer(VertexBufferHandle, TaskBufferPtr<[u8]>),

    CreateIndexBuffer(IndexBufferHandle, TaskBufferPtr<IndexBufferDescriptor>),
    UpdateIndexBuffer(IndexBufferHandle, TaskBufferPtr<[u8]>),
}

#[derive(Debug, Clone, Copy)]
pub enum PostFrameTask {
    DeleteView(ViewHandle),
    DeletePipeline(PipelineHandle),
    DeleteVertexBuffer(VertexBufferHandle),
    DeleteIndexBuffer(IndexBufferHandle),
}

#[derive(Debug, Clone, Copy)]
pub struct PipelineDescriptor {
    vs: TaskBufferPtr<str>,
    fs: TaskBufferPtr<str>,
    attributes: (u8, [VertexAttributeDesc; MAX_ATTRIBUTES]),
}

#[derive(Debug, Clone, Copy)]
pub struct VertexBufferDescriptor {
    layout: VertexLayout,
    size: u32,
    hint: ResourceHint,
    data: Option<TaskBufferPtr<[u8]>>,
}

#[derive(Debug, Clone, Copy)]
pub struct IndexBufferDescriptor {
    data: Option<TaskBufferPtr<[u8]>>,
}

pub struct Frame {
    pub pre: Vec<PreFrameTask>,
    pub post: Vec<PostFrameTask>,
    pub buf: TaskBuffer,
}

impl Frame {
    /// Creates a new frame with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Frame {
            pre: Vec::with_capacity(capacity),
            post: Vec::with_capacity(capacity),
            buf: TaskBuffer::with_capacity(capacity),
        }
    }
}

/// Where we store all the intermediate bytes.
pub struct TaskBuffer(Vec<u8>);

impl TaskBuffer {
    /// Creates a new task buffer with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        TaskBuffer(Vec::with_capacity(capacity))
    }

    pub fn extend<T>(&mut self, value: &T) -> TaskBufferPtr<T> where T: Copy {
        let data = unsafe {
            slice::from_raw_parts(value as *const T as *const u8, mem::size_of::<T>())  
        };

        self.0.extend_from_slice(data);
        TaskBufferPtr {
            position: (self.0.len() - data.len()) as u32,
            size: data.len() as u32,
            _phantom: PhantomData,
        }
    }

    /// Clones and appends all elements in a slice to the buffer.
    pub fn extend_from_slice<T>(&mut self, slice: &[T]) -> TaskBufferPtr<[T]>
        where T: Copy
    {
        let len = mem::size_of::<T>().wrapping_mul(slice.len());
        let u8_slice = unsafe { slice::from_raw_parts(slice.as_ptr() as *const u8, len) };
        self.0.extend_from_slice(u8_slice);
        TaskBufferPtr {
            position: (self.0.len() - len) as u32,
            size: len as u32,
            _phantom: PhantomData,
        }
    }

    /// Clones and append all bytes in a string slice to the buffer.
    pub fn extend_from_str<T>(&mut self, value: T) -> TaskBufferPtr<str> where T: Borrow<str>
    {
        let slice = self.extend_from_slice(value.borrow().as_bytes());
        TaskBufferPtr {
            position: slice.position,
            size: slice.size,
            _phantom: PhantomData,
        }
    }

    /// Returns reference to object indicated by `TaskBufferPtr`.
    #[inline]
    pub fn as_ref<T>(&self, ptr: TaskBufferPtr<T>) -> &T
        where T: Copy
    {
        let slice = self.as_bytes(ptr);
        assert_eq!(slice.len(), mem::size_of::<T>());
        unsafe { &*(slice.as_ptr() as *const _) }
    }

    /// Returns a object slice indicated by `TaskBufferPtr.
    #[inline]
    pub fn as_slice<T>(&self, ptr: TaskBufferPtr<[T]>) -> &[T]
        where T: Copy
    {
        let slice = self.as_bytes(ptr);
        let len = slice.len() / mem::size_of::<T>();
        assert_eq!(slice.len(), mem::size_of::<T>().wrapping_mul(len));
        unsafe { slice::from_raw_parts(slice.as_ptr() as *const T, len) }
    }

    /// Returns string slice indicated by `TaskBufferPtr`.
    #[inline]
    pub fn as_str(&self, ptr: TaskBufferPtr<str>) -> &str {
        str::from_utf8(self.as_bytes(ptr)).unwrap()
    }

    #[inline]
    pub fn as_bytes<T>(&self, slice:TaskBufferPtr<T>) -> &[u8] where T: ?Sized {
        &self.0[slice.position as usize..(slice.position + slice.size) as usize]
    }
}

/// A view into our `DataBuffer`, indicates where the object `T` stored.
#[derive(Debug)]
pub struct TaskBufferPtr<T> where T: ?Sized {
    position: u32,
    size: u32,
    _phantom: PhantomData<T>,
}

impl<T> Clone for TaskBufferPtr<T> where T: ?Sized {
    fn clone(&self) -> Self {
        TaskBufferPtr {
            position: self.position,
            size: self.size,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for TaskBufferPtr<T> where T: ?Sized {}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    struct UpdateViewRect {
        position: (u16, u16),
        size: (u16, u16),
    }

    #[test]
    fn buf() {
        let mut buffer = TaskBuffer::with_capacity(128);

        let mut uvp = UpdateViewRect::default();
        uvp.position = (256, 128);
        let slice_uvp = buffer.extend(&uvp);

        let int = 128 as u32;
        let slice_int = buffer.extend(&int);

        assert_eq!(*buffer.as_ref(slice_int), int);
        assert_eq!(*buffer.as_ref(slice_uvp), uvp);

        let arr = [1, 2, 3];
        let slice_arr = buffer.extend(&arr);
        assert_eq!(*buffer.as_ref(slice_arr), arr);

        let slice_arr_1_2 = buffer.extend_from_slice(&arr[0..2]);
        assert_eq!(buffer.as_slice(slice_arr_1_2), &arr[0..2]);

        let text = "string serialization";
        let slice_text = buffer.extend_from_str(text);
        assert_eq!(text, buffer.as_str(slice_text));
    }
}