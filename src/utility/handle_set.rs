use std::cmp::Ordering;
use std::collections::binary_heap::BinaryHeap;

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

/// `HandleSet` manages the manipulations of a `Handle` collection, which are
/// created with a continuous `index` field. It also have the ability to find
/// out the current status of a specified `Handle`.
pub struct HandleSet {
    versions: Vec<HandleIndex>,
    frees: BinaryHeap<InverseHandleIndex>,
}

impl HandleSet {
    /// Constructs a new, empty `HandleSet`.
    pub fn new() -> HandleSet {
        HandleSet {
            versions: Vec::new(),
            frees: BinaryHeap::new(),
        }
    }

    /// Constructs a new `HandleSet` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> HandleSet {
        let versions = Vec::with_capacity(capacity);
        let mut frees = BinaryHeap::with_capacity(capacity);
        for i in 0..versions.len() {
            frees.push(InverseHandleIndex(i as HandleIndex));
        }

        HandleSet {
            versions: versions,
            frees: frees,
        }
    }

    /// Creates a unused `Handle`.
    pub fn create(&mut self) -> Handle {
        if self.frees.len() > 0 {
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

    /// Returns true if this `Handle` was created by `HandleSet`, and has not been
    /// freed yet.
    pub fn is_alive(&self, handle: Handle) -> bool {
        let index = handle.index() as usize;
        (index < self.versions.len()) && ((self.versions[index] & 0x1) == 1) &&
        (self.versions[index] == handle.version())
    }

    /// Recycles the `Handle` index, and mark its version as dead.
    pub fn free(&mut self, handle: Handle) -> bool {
        if !self.is_alive(handle) {
            false
        } else {
            self.versions[handle.index() as usize] += 1;
            self.frees.push(InverseHandleIndex(handle.index()));
            true
        }
    }

    /// Returns the total number of alive handle in this `HandleSet`.
    #[inline]
    pub fn size(&self) -> usize {
        self.versions.len() - self.frees.len()
    }

    /// Returns an iterator over the `HandleSet`.
    #[inline]
    pub fn iter(&self) -> HandleIter {
        HandleIter {
            versions: &self.versions,
            start: 0,
            end: self.versions.len() as u32,
        }
    }
}

/// `HandleSet` with managed values `T`.
pub struct HandleSetWith<T: Sized> {
    handles: HandleSet,
    values: Vec<Option<T>>,
}

impl<T: Sized> HandleSetWith<T> {
    /// Constructs a new, empty `HandleSetWith`.
    pub fn new() -> Self {
        HandleSetWith {
            handles: HandleSet::new(),
            values: Vec::new(),
        }
    }

    /// Constructs a new `HandleSetWith` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        HandleSetWith {
            handles: HandleSet::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Creates a `T` and named it with `Handle`.
    pub fn create(&mut self, value: T) -> Handle {
        let handle = self.handles.create();

        if handle.index() >= self.values.len() as u32 {
            self.values.push(Some(value));
        } else {
            self.values[handle.index() as usize] = Some(value);
        }

        handle
    }

    /// Returns mutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        if self.handles.is_alive(handle) {
            self.values[handle.index() as usize].as_mut()
        } else {
            None
        }
    }

    /// Returns immutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get(&self, handle: Handle) -> Option<&T> {
        if self.handles.is_alive(handle) {
            self.values[handle.index() as usize].as_ref()
        } else {
            None
        }
    }

    /// Recycles the value with name `Handle`.
    #[inline]
    pub fn free(&mut self, handle: Handle) -> Option<T> {
        if self.handles.is_alive(handle) {
            let mut v = None;
            ::std::mem::swap(&mut v, &mut self.values[handle.index() as usize]);
            v
        } else {
            None
        }
    }

    /// Returns the total number of alive handle in this `HandleSetWith`.
    #[inline]
    pub fn size(&self) -> usize {
        self.handles.size()
    }
}

/// Immutable `HandleSet` iterator, this struct is created by `iter` method on `HandleSet`.
#[derive(Copy, Clone)]
pub struct HandleIter<'a> {
    versions: &'a Vec<HandleIndex>,
    start: HandleIndex,
    end: HandleIndex,
}

impl<'a> HandleIter<'a> {
    /// Divides iterator into two with specified stripe in the first `HandleIter`.
    pub fn split_with(&self, len: usize) -> (HandleIter<'a>, HandleIter<'a>) {
        let len = len as HandleIndex;
        let mid = if self.start + len >= self.end {
            self.end
        } else {
            self.start + len
        };

        let left = HandleIter {
            versions: self.versions,
            start: self.start,
            end: mid,
        };

        let right = HandleIter {
            versions: self.versions,
            start: mid,
            end: self.end,
        };

        (left, right)
    }

    /// Divides iterator into two at mid.
    /// The first will contain all indices from [start, mid) (excluding the index mid itself)
    /// and the second will contain all indices from [mid, end) (excluding the index end itself).
    pub fn split(&self) -> (HandleIter<'a>, HandleIter<'a>) {
        let mid = (self.end - self.start) / 2;
        self.split_with(mid as usize)
    }

    /// Returns the size of indices this iterator could reachs.
    pub fn len(&self) -> usize {
        (self.end - self.start) as usize
    }
}

impl<'a> Iterator for HandleIter<'a> {
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

#[cfg(test)]
mod test {
    use super::*;

    use std::cmp::min;
    use rand::{Rng, SeedableRng, XorShiftRng};

    #[test]
    fn handle_set() {
        let mut set = HandleSet::new();
        assert_eq!(set.size(), 0);

        // Spawn entities.
        let e1 = set.create();
        assert!(e1.is_valid());
        assert!(set.is_alive(e1));
        assert_eq!(set.size(), 1);

        let mut e2 = e1;
        assert!(set.is_alive(e2));
        assert_eq!(set.size(), 1);

        // Invalidate entities.
        e2.invalidate();
        assert!(!e2.is_valid());
        assert!(!set.is_alive(e2));
        assert!(set.is_alive(e1));

        // Free entities.
        let e2 = e1;
        set.free(e2);
        assert!(!set.is_alive(e2));
        assert!(!set.is_alive(e1));
        assert_eq!(set.size(), 0);
    }

    #[test]
    fn handle_set_with() {
        let mut set = HandleSetWith::<i32>::new();

        let e1 = set.create(3);
        assert_eq!(set.get(e1), Some(&3));
        assert_eq!(set.free(e1), Some(3));
        assert_eq!(set.get(e1), None);
        assert_eq!(set.free(e1), None)
    }

    #[test]
    fn index_reuse() {
        let mut set = HandleSet::new();

        assert_eq!(set.size(), 0);

        let mut v = vec![];
        for _ in 0..10 {
            v.push(set.create());
        }

        assert_eq!(set.size(), 10);
        for e in v.iter() {
            set.free(*e);
        }

        for _ in 0..10 {
            let e = set.create();
            assert!((*e as usize) < v.len());
            assert!(v[*e as usize].version() != e.version());
        }
    }

    #[test]
    fn index_compact_reuse() {
        let mut generator = XorShiftRng::from_seed([0, 1, 2, 3]);
        let mut set = HandleSet::new();

        let mut v = vec![];
        for _ in 0..5 {
            for _ in 0..50 {
                v.push(set.create());
            }

            let size = v.len() / 2;
            for _ in 0..size {
                let len = v.len();
                set.free(v.swap_remove(generator.next_u32() as usize % len));
            }
        }

        for i in v {
            set.free(i);
        }

        for index in 0..50 {
            let handle = set.create();
            assert_eq!(handle.index(), index);
        }
    }

    #[test]
    fn iter() {
        let mut set = HandleSet::new();
        let mut v = vec![];

        for m in 2..3 {
            for _ in 0..10 {
                v.push(set.create())
            }

            for i in 0..10 {
                if i % m == 0 {
                    let index = i % v.len();
                    set.free(v[index]);
                    v.remove(index);
                }
            }
        }

        v.sort_by(|lhs, rhs| lhs.index().cmp(&rhs.index()));
        let mut iter = set.iter();
        let test_split_with = |stride| {
            let iter = set.iter();
            let (mut s1, mut s2) = iter.split_with(stride);
            assert_eq!(s1.len(), min(stride, iter.len()));
            assert_eq!(s2.len(), iter.len() - min(stride, iter.len()));

            for handle in &v {
                if let Some(v) = s1.next() {
                    assert_eq!(*handle, v);
                } else {
                    assert_eq!(*handle, s2.next().unwrap());
                }
            }
        };

        test_split_with(0);
        test_split_with(1);
        test_split_with(iter.len() - 1);
        test_split_with(iter.len());
        test_split_with(iter.len() + 1);
        test_split_with(iter.len() * 2);

        for handle in &v {
            assert_eq!(*handle, iter.next().unwrap());
        }
    }
}
