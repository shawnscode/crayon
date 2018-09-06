//! Small vector optimization that stores up to a small number of items on the stack.

use std::ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeTo};
use std::{fmt, slice};

#[derive(Clone)]
enum PrivateVariantVec<T: Array> {
    Stack(u8, T),
    Unconstraint(Vec<T::Item>),
}

/// Small vector optimization that stores up to a small number of items on the stack.
#[derive(Clone)]
pub struct VariantVec<T: Array>(PrivateVariantVec<T>);

impl<T: Array> Deref for VariantVec<T> {
    type Target = [T::Item];

    #[inline]
    fn deref(&self) -> &[T::Item] {
        self.as_slice()
    }
}

impl<T: Array> DerefMut for VariantVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T::Item] {
        self.as_mut_slice()
    }
}

impl<T: Array + Default> Default for VariantVec<T> {
    fn default() -> Self {
        VariantVec(PrivateVariantVec::Stack(0, T::default()))
    }
}

impl<T: Array + Default> VariantVec<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: Array<Item = U> + fmt::Debug, U: Copy + fmt::Debug> fmt::Debug for VariantVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            PrivateVariantVec::Stack(len, ref v) => unsafe {
                let slice = slice::from_raw_parts(v.ptr(), len as usize);
                write!(f, "VariantVec::Stack({:?})", slice)
            },
            PrivateVariantVec::Unconstraint(ref v) => {
                write!(f, "VariantVec::Unconstraint({:?})", v)
            }
        }
    }
}

impl<T: Array> VariantVec<T> {
    /// Gets the len of vec.
    #[inline]
    pub fn len(&self) -> usize {
        match self.0 {
            PrivateVariantVec::Stack(len, _) => len as usize,
            PrivateVariantVec::Unconstraint(ref v) => v.len(),
        }
    }

    /// Checks if the `VariantVec` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Extracts a slice containing the entire vector.
    #[inline]
    pub fn as_slice(&self) -> &[T::Item] {
        match self.0 {
            PrivateVariantVec::Stack(len, ref v) => unsafe {
                slice::from_raw_parts(v.ptr(), len as usize)
            },
            PrivateVariantVec::Unconstraint(ref v) => v.as_slice(),
        }
    }

    /// Extracts a mutable slice of the entire vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T::Item] {
        match self.0 {
            PrivateVariantVec::Stack(len, ref mut v) => unsafe {
                slice::from_raw_parts_mut(v.ptr_mut(), len as usize)
            },
            PrivateVariantVec::Unconstraint(ref mut v) => v.as_mut_slice(),
        }
    }

    /// Appends an element to the back of a collection.
    #[inline]
    pub fn push(&mut self, value: T::Item) {
        let update = match self.0 {
            PrivateVariantVec::Stack(ref mut len, ref mut v) => unsafe {
                let index = *len as usize;
                if index >= T::size() {
                    let mut w = Vec::from(slice::from_raw_parts(v.ptr(), T::size()));
                    w.push(value);
                    Some(PrivateVariantVec::Unconstraint(w))
                } else {
                    slice::from_raw_parts_mut(v.ptr_mut(), T::size())[index] = value;
                    *len = *len + 1;
                    None
                }
            },
            PrivateVariantVec::Unconstraint(ref mut v) => {
                v.push(value);
                None
            }
        };

        if let Some(v) = update {
            self.0 = v;
        }
    }
}

macro_rules! impl_index {
    ($index_type:ty, $output_type:ty) => {
        impl<T: Array> Index<$index_type> for VariantVec<T> {
            type Output = $output_type;
            #[inline]
            fn index(&self, index: $index_type) -> &$output_type {
                &(&**self)[index]
            }
        }

        impl<T: Array> IndexMut<$index_type> for VariantVec<T> {
            #[inline]
            fn index_mut(&mut self, index: $index_type) -> &mut $output_type {
                &mut (&mut **self)[index]
            }
        }
    };
}

impl_index!(usize, T::Item);
impl_index!(Range<usize>, [T::Item]);
impl_index!(RangeFrom<usize>, [T::Item]);
impl_index!(RangeTo<usize>, [T::Item]);
impl_index!(RangeFull, [T::Item]);

/// Types that can be used as the backing store for a SmallVec
pub unsafe trait Array {
    /// The type of the array's elements.
    type Item: Copy;

    /// Returns the number of items the array can hold.
    fn size() -> usize;
    /// Returns a pointer to the first element of the array.
    fn ptr(&self) -> *const Self::Item;
    /// Returns a mutable pointer to the first element of the array.
    fn ptr_mut(&mut self) -> *mut Self::Item;
}

macro_rules! impl_array(
    ($($size:expr),+) => {
        $(
            unsafe impl<T: Copy> Array for [T; $size] {
                type Item = T;

                fn size() -> usize { $size }
                fn ptr(&self) -> *const T { self.as_ptr() }
                fn ptr_mut(&mut self) -> *mut T { self.as_mut_ptr() }
            }
        )+
    }
);

impl_array!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 20, 24, 32, 36);

#[cfg(test)]
mod test {
    use super::*;

    impl<T: Array> VariantVec<T> {
        #[inline]
        fn is_stack(&self) -> bool {
            match self.0 {
                PrivateVariantVec::Stack(_, _) => true,
                PrivateVariantVec::Unconstraint(_) => false,
            }
        }
    }

    #[test]
    fn basic() {
        let mut v = VariantVec::<[_; 4]>::new();
        assert!(v.is_stack());
        v.push(0);
        assert!(v.is_stack());
        v.push(2);
        assert!(v.is_stack());
        v.push(4);
        assert!(v.is_stack());
        v.push(8);
        assert!(v.is_stack());
        assert_eq!(v.len(), 4);

        v.push(16);
        assert!(!v.is_stack());
    }

    #[test]
    fn index() {
        let mut v = VariantVec::<[_; 4]>::new();
        v.push(0);
        v.push(2);

        assert_eq!(v[0], 0);
        assert_eq!(v[1], 2);
    }
}
