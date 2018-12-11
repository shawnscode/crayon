use std::cmp::Ordering;
use std::collections::binary_heap::BinaryHeap;
use std::marker::PhantomData;

use super::handle::{HandleIndex, HandleLike};

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
pub struct HandlePool<T: HandleLike> {
    versions: Vec<HandleIndex>,
    frees: BinaryHeap<InverseHandleIndex>,
    _marker: PhantomData<T>,
}

impl<T: HandleLike> Default for HandlePool<T> {
    fn default() -> Self {
        HandlePool {
            versions: Vec::new(),
            frees: BinaryHeap::new(),
            _marker: PhantomData::default(),
        }
    }
}

impl<T: HandleLike> HandlePool<T> {
    /// Constructs a new, empty `HandlePool`.
    pub fn new() -> HandlePool<T> {
        Default::default()
    }

    /// Constructs a new `HandlePool` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> HandlePool<T> {
        let versions = Vec::with_capacity(capacity);
        let mut frees = BinaryHeap::with_capacity(capacity);
        for i in 0..versions.len() {
            frees.push(InverseHandleIndex(i as HandleIndex));
        }

        HandlePool {
            versions,
            frees,
            _marker: PhantomData::default(),
        }
    }

    /// Creates a unused `Handle`.
    pub fn create(&mut self) -> T {
        if !self.frees.is_empty() {
            // If we have available free slots.
            let index = self.frees.pop().unwrap().0 as usize;
            self.versions[index] += 1;
            T::new(index as HandleIndex, self.versions[index])
        } else {
            // Or we just spawn a new index and corresponding version.
            self.versions.push(1);
            T::new(self.versions.len() as HandleIndex - 1, 1)
        }
    }

    /// Returns true if this `Handle` was created by `HandlePool`, and has not been
    /// freed yet.
    pub fn contains(&self, handle: T) -> bool {
        let index = handle.index() as usize;
        (index < self.versions.len())
            && ((self.versions[index] & 0x1) == 1)
            && (self.versions[index] == handle.version())
    }

    /// Recycles the `Handle` index, and mark its version as dead.
    pub fn free(&mut self, handle: T) -> bool {
        if !self.contains(handle) {
            false
        } else {
            self.versions[handle.index() as usize] += 1;
            self.frees.push(InverseHandleIndex(handle.index()));
            true
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e such that predicate(&T) returns false.
    pub fn retain<P>(&mut self, mut predicate: P)
    where
        P: FnMut(T) -> bool,
    {
        unsafe {
            for i in 0..self.versions.len() {
                let v = *self.versions.get_unchecked(i);
                if v & 0x1 == 1 && !predicate(T::new(i as u32, v)) {
                    self.versions[i] += 1;
                    self.frees.push(InverseHandleIndex(i as u32));
                }
            }
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

    /// An iterator visiting all the handles.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }
}

impl<'a, T: HandleLike> IntoIterator for &'a HandlePool<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<'a, T: HandleLike> IntoIterator for &'a mut HandlePool<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

/// Immutable `HandlePool` iterator, this struct is created by `iter` method on `HandlePool`.
#[derive(Copy, Clone)]
pub struct Iter<'a, T: HandleLike> {
    versions: &'a [HandleIndex],
    start: HandleIndex,
    end: HandleIndex,
    _marker: PhantomData<T>,
}

impl<'a, T: HandleLike> Iter<'a, T> {
    fn new(handles: &'a HandlePool<T>) -> Self {
        Iter {
            versions: &handles.versions,
            start: 0,
            end: handles.versions.len() as u32,
            _marker: handles._marker,
        }
    }

    /// Divides iterator into two with specified stripe in the first `Iter`.
    pub fn split_at(&self, len: usize) -> (Iter<'a, T>, Iter<'a, T>) {
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
            _marker: self._marker,
        };

        let right = Iter {
            versions: self.versions,
            start: mid,
            end: self.end,
            _marker: self._marker,
        };

        (left, right)
    }

    /// Divides iterator into two at mid.
    ///
    /// The first will contain all indices from [start, mid) (excluding the index mid itself)
    /// and the second will contain all indices from [mid, end) (excluding the index end itself).
    #[inline]
    pub fn split(&self) -> (Iter<'a, T>, Iter<'a, T>) {
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

impl<'a, T: HandleLike> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        unsafe {
            for i in self.start..self.end {
                let v = self.versions.get_unchecked(i as usize);
                if v & 0x1 == 1 {
                    self.start = i + 1;
                    return Some(T::new(i, *v));
                }
            }
        }

        None
    }
}

impl<'a, T: HandleLike> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        unsafe {
            for i in (self.start..self.end).rev() {
                let v = self.versions.get_unchecked(i as usize);
                if v & 0x1 == 1 {
                    self.end = i;
                    return Some(T::new(i, *v));
                }
            }
        }

        None
    }
}
