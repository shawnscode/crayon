use std::marker::PhantomData;
use std::borrow::Borrow;
use std::str;
use std::slice;
use std::mem;

use super::*;

#[derive(Debug, Clone, Copy)]
pub enum FrameTask {
    CreateView(DataSlice<CreateView>),
    UpdateViewRect(DataSlice<UpdateViewRect>),
    UpdateViewScissor(DataSlice<UpdateViewRect>),
    UpdateViewClear(DataSlice<UpdateViewClear>),
    DeleteView(DataSlice<DeleteView>),

    CreatePipeline(DataSlice<CreatePipeline>),
    UpdatePipelineState(DataSlice<UpdatePipelineState>),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CreateView {
    handle: ViewHandle,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UpdateViewRect {
    handle: ViewHandle,
    position: (u16, u16),
    size: (u16, u16),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateViewClear {
    handle: ViewHandle,
    color: Option<u32>,
    depth: Option<f32>,
    stencil: Option<i32>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteView {
    handle: ViewHandle,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct CreatePipeline {
    handle: PipelineHandle,
    vs: DataSlice<str>,
    fs: DataSlice<str>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdatePipelineState {
    handle: PipelineHandle,
    state: RenderState,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdatePipelineUniforms {
    handle: PipelineHandle,
    uniforms: DataSlice<[UpdateUniform]>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UniformVariable {
    Vector1([f32; 1]),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    Matrix2([f32; 4]),
    Matrix3([f32; 9]),
    Matrix4([f32; 16]),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateUniform {
    name: [u8; 32],
    format: u8,
    data: DataSlice<[f32]>,
}

impl UpdateUniform {
    pub fn encode<T>(buf: &mut DataBuffer, name: T, variable: &UniformVariable) -> UpdateUniform
        where T: Borrow<str>
    {
        let (format, data) = variable.encode();
        UpdateUniform {
            name: buf.extend_from_str(name.borrow()),
            format: format,
            data: buf.extend_from_slice(data),
        }
    }
}

/// Where we store all the intermediate bytes.
#[derive(Debug, Default)]
pub struct DataBuffer(Vec<u8>);

/// A view into our `DataBuffer`, indicates where the object `T` stored.
#[derive(Debug)]
pub struct DataSlice<T> where T : ?Sized {
    offset: u32,
    size: u32,
    phantom: PhantomData<T>,
}

impl<T> Clone for DataSlice<T> where T: ?Sized {
    fn clone(&self) -> Self {
        DataSlice {
            offset: self.offset,
            size: self.size,
            phantom: Default::default(),
        }
    }
}

impl<T> Copy for DataSlice<T> where T: ?Sized {}

impl<T> Default for DataSlice<T> where T: ?Sized {
    fn default() -> Self {
        DataSlice {
            offset: 0, size: 0, phantom: Default::default()
        }
    }
}

impl DataBuffer {
    /// Creates a new data buffer with specified capacity.
    pub fn with_capacity(capacity: usize) -> DataBuffer {
        DataBuffer(Vec::with_capacity(capacity))
    }

    /// Extend data buffer with specified copyable value, which implicates that
    /// types whose values can be duplicated simply by copying bits.
    pub fn extend<T>(&mut self, value: &T) -> DataSlice<T>
        where T: Copy
    {
        let data =
            unsafe { slice::from_raw_parts(value as *const T as *const u8, mem::size_of::<T>()) };

        self.0.extend_from_slice(data);
        DataSlice {
            offset: (self.0.len() - data.len()) as u32,
            size: data.len() as u32,
            phantom: PhantomData::default(),
        }
    }

    /// Clones and appends all elements in a slice to the buffer.
    pub fn extend_from_slice<T>(&mut self, slice: &[T]) -> DataSlice<[T]>
        where T: Copy
    {
        let len = mem::size_of::<T>().wrapping_mul(slice.len());
        let u8_slice = unsafe { slice::from_raw_parts(slice.as_ptr() as *const u8, len) };
        self.0.extend_from_slice(u8_slice);
        DataSlice {
            offset: (self.0.len() - len) as u32,
            size: len as u32,
            phantom: PhantomData::default(),
        }
    }

    /// CLones and append all bytes in a string slice to the buffer.
    pub fn extend_from_str<T>(&mut self, value: T) -> DataSlice<str> where T: Borrow<str>
    {
        let slice = self.extend_from_slice(value.borrow().as_bytes());
        DataSlice {
            offset: slice.offset,
            size: slice.size,
            phantom: PhantomData::default(),
        }
    }

    /// Returns reference to object indicated by `DataSlice`.
    #[inline]
    pub fn as_ref<T>(&self, ptr: DataSlice<T>) -> &T
        where T: Copy
    {
        let slice = self.as_u8_slice(ptr);
        assert_eq!(slice.len(), mem::size_of::<T>());
        unsafe { &*(slice.as_ptr() as *const _) }
    }

    /// Returns a object slice indicated by `DataSlice.
    #[inline]
    pub fn as_slice<T>(&self, ptr: DataSlice<[T]>) -> &[T]
        where T: Copy
    {
        let slice = self.as_u8_slice(ptr);
        let len = slice.len() / mem::size_of::<T>();
        assert_eq!(slice.len(), mem::size_of::<T>().wrapping_mul(len));
        unsafe { slice::from_raw_parts(slice.as_ptr() as *const T, len) }
    }

    /// Returns string slice indicated by `DataSlice`.
    #[inline]
    pub fn as_str(&self, ptr: DataSlice<str>) -> &str {
        str::from_utf8(self.as_u8_slice(ptr)).unwrap()
    }

    #[inline]
    fn as_u8_slice<T>(&self, slice:DataSlice<T>) -> &[u8] where T: ?Sized {
        &self.0[slice.offset as usize..(slice.offset + slice.size) as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn buffer() {
        let mut buffer = DataBuffer::with_capacity(128);

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