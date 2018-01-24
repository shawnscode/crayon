use std::borrow::Borrow;
use super::{Handle, HandleIter, HandlePool};

/// A named object collections. Every time u create or free a handle, a
/// attached instance `T` will be created/ freed.
pub struct HandleObjectPool<T: Sized> {
    handles: HandlePool,
    entries: Vec<Option<T>>,
}

impl<T: Sized> HandleObjectPool<T> {
    /// Constructs a new, empty `ObjectPool`.
    pub fn new() -> Self {
        HandleObjectPool {
            handles: HandlePool::new(),
            entries: Vec::new(),
        }
    }

    /// Constructs a new `ObjectPool` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        HandleObjectPool {
            handles: HandlePool::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Creates a `T` and named it with `Handle`.
    pub fn create(&mut self, value: T) -> Handle {
        let handle = self.handles.create();

        if handle.index() >= self.entries.len() as u32 {
            self.entries.push(Some(value));
        } else {
            self.entries[handle.index() as usize] = Some(value);
        }

        handle
    }

    /// Returns mutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        if self.handles.is_alive(handle) {
            self.entries[handle.index() as usize].as_mut()
        } else {
            None
        }
    }

    /// Returns immutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get<H>(&self, handle: H) -> Option<&T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        if self.handles.is_alive(handle) {
            self.entries[handle.index() as usize].as_ref()
        } else {
            None
        }
    }

    /// Returns true if this `Handle` was created by `ObjectPool`, and has not been
    /// freed yet.
    #[inline]
    pub fn is_alive<H>(&self, handle: H) -> bool
    where
        H: Borrow<Handle>,
    {
        self.handles.is_alive(handle)
    }

    /// Recycles the value with name `Handle`.
    #[inline]
    pub fn free<H>(&mut self, handle: H) -> Option<T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        if self.handles.free(handle) {
            let mut v = None;
            ::std::mem::swap(&mut v, &mut self.entries[handle.index() as usize]);
            v
        } else {
            None
        }
    }

    /// Remove all objects matching with `predicate` from pool incrementally.
    pub fn free_if<P>(&mut self, predicate: P) -> FreeIter<T, P>
    where
        P: FnMut(&T) -> bool,
    {
        FreeIter {
            index: 0,
            entries: &mut self.entries[..],
            handles: &mut self.handles,
            predicate: predicate,
        }
    }

    /// Returns the total number of alive handle in this `ObjectPool`.
    #[inline]
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Returns an iterator over the `ObjectPool`.
    #[inline]
    pub fn iter(&self) -> HandleIter {
        self.handles.iter()
    }
}

pub struct FreeIter<'a, T: 'a, P>
where
    P: FnMut(&T) -> bool,
{
    index: usize,
    entries: &'a mut [Option<T>],
    handles: &'a mut HandlePool,
    predicate: P,
}

impl<'a, T: 'a, P> Iterator for FreeIter<'a, T, P>
where
    P: FnMut(&T) -> bool,
{
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            for i in self.index..self.entries.len() {
                let v = self.entries.get_unchecked_mut(i);

                let free = if let &mut Some(ref payload) = v {
                    (self.predicate)(payload)
                } else {
                    false
                };

                if free {
                    let handle = self.handles.free_at(i).unwrap();
                    *v = None;
                    return Some(handle);
                }
            }

            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut set = HandleObjectPool::<i32>::new();

        let e1 = set.create(3);
        assert_eq!(set.get(e1), Some(&3));
        assert_eq!(set.len(), 1);
        assert_eq!(set.free(e1), Some(3));
        assert_eq!(set.len(), 0);
        assert_eq!(set.get(e1), None);
        assert_eq!(set.free(e1), None);
        assert_eq!(set.len(), 0);
    }
}
