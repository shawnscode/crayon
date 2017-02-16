use std::mem;
use std::slice;

/// A marker trait indicating that a type is plain-old-data. A pod type does not have
/// invalid bit patterns and can be safely created from arbitrary bit pattern.
pub unsafe trait Pod: Sized {
    /// Borrows the POD as a byte slice
    #[inline]
    fn as_slice<'a>(&'a self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self as *const Self as *const u8, mem::size_of::<Self>()) }
    }

    /// Borrows a new instance of the POD from a byte slice
    ///
    /// # Panics
    ///
    /// Panics if `slice.len()` is not the same as the type's size
    #[inline]
    fn from_slice<'a>(raw: &'a [u8]) -> &'a Self {
        assert_eq!(raw.len(), mem::size_of::<Self>());
        unsafe { &*(raw.as_ptr() as *const _) }
    }
}

macro_rules! impl_pod {
    ( ty = $($ty:ty)* ) => { $( unsafe impl Pod for $ty {} )* };
    ( ar = $($tt:expr)* ) => { $( unsafe impl<T: Pod> Pod for [T; $tt] {} )* };
}

impl_pod! { ty = isize usize i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 }
impl_pod! { ar = 0 1 2 3 4 5 6 7 8 9 10 11 12 }
impl_pod! { ar = 13 14 15 16 17 18 19 20 21 22 23 24 }
impl_pod! { ar = 25 26 27 28 29 30 31 32 33 34 35 36 }
unsafe impl<T: Pod, U: Pod> Pod for (T, U) {}

/// Cast arbitrary Pod slice to another arbitrary Pod slice.
pub fn cast_slice<A: Pod, B: Pod>(slice: &[A]) -> &[B] {
    use std::slice;

    let raw_len = mem::size_of::<A>().wrapping_mul(slice.len());
    let len = raw_len / mem::size_of::<B>();
    assert_eq!(raw_len, mem::size_of::<B>().wrapping_mul(len));
    unsafe { slice::from_raw_parts(slice.as_ptr() as *const B, len) }
}

#[cfg(test)]
mod test {}
