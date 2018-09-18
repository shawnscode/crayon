use std::borrow::Borrow;
use std::marker::PhantomData;
use std::{mem, slice, str};

/// Where we store all the intermediate bytes.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DataBuffer(Vec<u8>);

impl DataBuffer {
    /// Creates a new and emplty `DataBuffer`.
    pub fn new() -> Self {
        DataBuffer(Vec::new())
    }

    /// Creates a new task buffer with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        DataBuffer(Vec::with_capacity(capacity))
    }

    pub fn clear(&mut self) {
        unsafe {
            self.0.set_len(0);
        }
    }

    pub fn extend<T>(&mut self, value: &T) -> DataBufferPtr<T>
    where
        T: Copy,
    {
        let data =
            unsafe { slice::from_raw_parts(value as *const T as *const u8, mem::size_of::<T>()) };

        self.0.extend_from_slice(data);

        DataBufferPtr {
            position: (self.0.len() - data.len()) as u32,
            size: data.len() as u32,
            _phantom: PhantomData,
        }
    }

    /// Clones and appends all elements in a slice to the buffer.
    pub fn extend_from_slice<T>(&mut self, slice: &[T]) -> DataBufferPtr<[T]>
    where
        T: Copy,
    {
        let len = mem::size_of::<T>().wrapping_mul(slice.len());
        let u8_slice = unsafe { slice::from_raw_parts(slice.as_ptr() as *const u8, len) };
        self.0.extend_from_slice(u8_slice);
        DataBufferPtr {
            position: (self.0.len() - len) as u32,
            size: len as u32,
            _phantom: PhantomData,
        }
    }

    /// Clones and append all bytes in a string slice to the buffer.
    pub fn extend_from_str<T>(&mut self, value: T) -> DataBufferPtr<str>
    where
        T: Borrow<str>,
    {
        let slice = self.extend_from_slice(value.borrow().as_bytes());
        DataBufferPtr {
            position: slice.position,
            size: slice.size,
            _phantom: PhantomData,
        }
    }

    /// Returns reference to object indicated by `DataBufferPtr`.
    #[inline]
    pub fn as_ref<T>(&self, ptr: DataBufferPtr<T>) -> &T
    where
        T: Copy,
    {
        let slice = self.as_bytes(ptr);
        assert_eq!(slice.len(), mem::size_of::<T>());
        unsafe { &*(slice.as_ptr() as *const _) }
    }

    /// Returns a object slice indicated by `DataBufferPtr.
    #[inline]
    pub fn as_slice<T>(&self, ptr: DataBufferPtr<[T]>) -> &[T]
    where
        T: Copy,
    {
        let slice = self.as_bytes(ptr);
        let len = slice.len() / mem::size_of::<T>();
        assert_eq!(slice.len(), mem::size_of::<T>().wrapping_mul(len));
        unsafe { slice::from_raw_parts(slice.as_ptr() as *const T, len) }
    }

    /// Returns string slice indicated by `DataBufferPtr`.
    #[inline]
    pub fn as_str(&self, ptr: DataBufferPtr<str>) -> &str {
        str::from_utf8(self.as_bytes(ptr)).unwrap()
    }

    #[inline]
    pub fn as_bytes<T>(&self, slice: DataBufferPtr<T>) -> &[u8]
    where
        T: ?Sized,
    {
        &self.0[slice.position as usize..(slice.position + slice.size) as usize]
    }
}

/// A view into our `DataBuffer`, indicates where the object `T` stored.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct DataBufferPtr<T>
where
    T: ?Sized,
{
    position: u32,
    size: u32,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized> Clone for DataBufferPtr<T> {
    fn clone(&self) -> Self {
        DataBufferPtr {
            position: self.position,
            size: self.size,
            _phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Copy for DataBufferPtr<T> {}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    struct UpdateSurfaceRect {
        position: (u16, u16),
        size: (u16, u16),
    }

    #[test]
    fn buf() {
        let mut buffer = DataBuffer::with_capacity(128);

        let mut uvp = UpdateSurfaceRect::default();
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
