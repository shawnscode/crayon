use std::cmp::Ordering;
use std::collections::binary_heap::BinaryHeap;
use std::borrow::Borrow;

use super::{Handle, HandleIndex};

#[derive(PartialEq, Eq)]
struct InverseHandleIndex(HandleIndex);

impl PartialOrd for InverseHandleIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for InverseHandleIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// `HandlePool` manages the manipulations of a `Handle` collection, which are
/// created with a continuous `index` field. It also have the ability to find
/// out the current status of a specified `Handle`.
#[derive(Default)]
pub struct HandlePool {
    versions: Vec<HandleIndex>,
    frees: BinaryHeap<InverseHandleIndex>,
}

impl HandlePool {
    /// Constructs a new, empty `HandlePool`.
    pub fn new() -> HandlePool {
        HandlePool::default()
    }

    /// Constructs a new `HandlePool` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> HandlePool {
        let versions = Vec::with_capacity(capacity);
        let mut frees = BinaryHeap::with_capacity(capacity);
        for i in 0..versions.len() {
            frees.push(InverseHandleIndex(i as HandleIndex));
        }

        HandlePool {
            versions: versions,
            frees: frees,
        }
    }

    /// Creates a unused `Handle`.
    pub fn create(&mut self) -> Handle {
        if !self.frees.is_empty() {
            // If we have available free slots.
            let index = self.frees.pop().unwrap().0 as usize;
            self.versions[index] += 1;
            Handle::new(index as HandleIndex, self.versions[index])
        } else {
            // Or we just spawn a new index and corresponding version.
            self.versions.push(1);
            Handle::new(self.versions.len() as HandleIndex - 1, 1)
        }
    }

    /// Returns true if this `Handle` was created by `HandlePool`, and has not been
    /// freed yet.
    pub fn is_alive<T>(&self, handle: T) -> bool
    where
        T: Borrow<Handle>,
    {
        let handle = handle.borrow();
        let index = handle.index() as usize;
        self.is_alive_at(index) && (self.versions[index] == handle.version())
    }

    #[inline]
    fn is_alive_at(&self, index: usize) -> bool {
        (index < self.versions.len()) && ((self.versions[index] & 0x1) == 1)
    }

    /// Recycles the `Handle` index, and mark its version as dead.
    pub fn free<T>(&mut self, handle: T) -> bool
    where
        T: Borrow<Handle>,
    {
        let handle = handle.borrow();
        if !self.is_alive(handle) {
            false
        } else {
            self.versions[handle.index() as usize] += 1;
            self.frees.push(InverseHandleIndex(handle.index()));
            true
        }
    }

    /// Recycles the `Handle` index, and mark its version as dead.
    pub fn free_at(&mut self, index: usize) -> Option<Handle> {
        if !self.is_alive_at(index) {
            None
        } else {
            self.versions[index] += 1;
            self.frees.push(InverseHandleIndex(index as HandleIndex));
            Some(Handle::new(index as HandleIndex, self.versions[index] - 1))
        }
    }

    /// Clears the `HandlePool`, removing all versions. Keeps the allocated memory
    /// for reuse.
    pub fn clear(&mut self) {
        self.frees.clear();
        self.versions.clear();
    }

    /// Returns the total number of alive handle in this `HandlePool`.
    #[inline]
    pub fn len(&self) -> usize {
        self.versions.len() - self.frees.len()
    }

    /// Checks if the pool is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the `HandlePool`.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }
}

impl<'a> IntoIterator for &'a HandlePool {
    type Item = Handle;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<'a> IntoIterator for &'a mut HandlePool {
    type Item = Handle;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

/// Immutable `HandlePool` iterator, this struct is created by `iter` method on `HandlePool`.
#[derive(Copy, Clone)]
pub struct Iter<'a> {
    versions: &'a [HandleIndex],
    start: HandleIndex,
    end: HandleIndex,
}

impl<'a> Iter<'a> {
    fn new(handles: &'a HandlePool) -> Self {
        Iter {
            versions: &handles.versions,
            start: 0,
            end: handles.versions.len() as u32,
        }
    }

    /// Divides iterator into two with specified stripe in the first `Iter`.
    pub fn split_at(&self, len: usize) -> (Iter<'a>, Iter<'a>) {
        let len = len as HandleIndex;
        let mid = if self.start + len >= self.end {
            self.end
        } else {
            self.start + len
        };

        let left = Iter {
            versions: self.versions,
            start: self.start,
            end: mid,
        };

        let right = Iter {
            versions: self.versions,
            start: mid,
            end: self.end,
        };

        (left, right)
    }

    /// Divides iterator into two at mid.
    ///
    /// The first will contain all indices from [start, mid) (excluding the index mid itself)
    /// and the second will contain all indices from [mid, end) (excluding the index end itself).
    #[inline]
    pub fn split(&self) -> (Iter<'a>, Iter<'a>) {
        let mid = (self.end - self.start) / 2;
        self.split_at(mid as usize)
    }

    /// Returns the size of indices this iterator could reachs.
    #[inline]
    pub fn len(&self) -> usize {
        (self.end - self.start) as usize
    }

    /// Checks if the iterator is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Handle;

    fn next(&mut self) -> Option<Handle> {
        unsafe {
            for i in self.start..self.end {
                let v = self.versions.get_unchecked(i as usize);
                if v & 0x1 == 1 {
                    self.start = i + 1;
                    return Some(Handle::new(i, *v));
                }
            }
        }
        None
    }
}
