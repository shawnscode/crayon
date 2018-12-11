use super::handle::HandleLike;
use super::handle_pool::HandlePool;

/// A named object collections. Every time u create or free a handle, a
/// attached instance `T` will be created/ freed.
pub struct ObjectPool<H: HandleLike, T: Sized> {
    handles: HandlePool<H>,
    entries: Vec<T>,
}

impl<H: HandleLike, T: Sized> Default for ObjectPool<H, T> {
    fn default() -> Self {
        ObjectPool {
            handles: HandlePool::new(),
            entries: Vec::new(),
        }
    }
}

impl<H: HandleLike, T: Sized> ObjectPool<H, T> {
    /// Constructs a new, empty `ObjectPool`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Constructs a new `ObjectPool` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        ObjectPool {
            handles: HandlePool::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Creates a `T` and named it with `Handle`.
    pub fn create(&mut self, mut value: T) -> H {
        let handle = self.handles.create();

        if handle.index() >= self.entries.len() as u32 {
            self.entries.push(value);
        } else {
            ::std::mem::swap(&mut value, &mut self.entries[handle.index() as usize]);
            ::std::mem::forget(value);
        }

        handle
    }

    /// Returns mutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get_mut(&mut self, handle: H) -> Option<&mut T> {
        if self.handles.contains(handle) {
            unsafe { Some(self.entries.get_unchecked_mut(handle.index() as usize)) }
        } else {
            None
        }
    }

    /// Returns immutable reference to internal value with name `Handle`.
    #[inline]
    pub fn get(&self, handle: H) -> Option<&T> {
        if self.handles.contains(handle) {
            unsafe { Some(self.entries.get_unchecked(handle.index() as usize)) }
        } else {
            None
        }
    }

    /// Returns true if this `Handle` was created by `ObjectPool`, and has not been
    /// freed yet.
    #[inline]
    pub fn contains(&self, handle: H) -> bool {
        self.handles.contains(handle)
    }

    /// Recycles the value with name `Handle`.
    #[inline]
    pub fn free(&mut self, handle: H) -> Option<T> {
        if self.handles.free(handle) {
            unsafe {
                let mut v = ::std::mem::uninitialized();
                ::std::mem::swap(&mut v, &mut self.entries[handle.index() as usize]);
                Some(v)
            }
        } else {
            None
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all objects such that f(k, &mut v) returns false.
    pub fn retain<P>(&mut self, mut predicate: P)
    where
        P: FnMut(H, &mut T) -> bool,
    {
        let entries = &mut self.entries;
        self.handles.retain(|handle| unsafe {
            let mut v = entries.get_unchecked_mut(handle.index() as usize);
            if predicate(handle, v) {
                true
            } else {
                #[allow(clippy::invalid_ref)]
                std::mem::swap(&mut v, &mut std::mem::uninitialized());
                false
            }
        });
    }

    /// Returns the total number of alive handle in this `ObjectPool`.
    #[inline]
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Checks if the pool is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// an iterator visiting all key-value pairs in order. the iterator element type is (h, &t).
    #[inline]
    pub fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = (H, &T)> + 'a {
        self.handles
            .iter()
            .map(move |v| unsafe { (v, self.entries.get_unchecked(v.index() as usize)) })
    }

    /// an iterator visiting all key-value pairs in order. the iterator element type is (h, &mut t).
    #[inline]
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (H, &mut T)> {
        let entries = &mut self.entries;
        self.handles.iter().map(move |v| unsafe {
            let w = entries.get_unchecked_mut(v.index() as usize);
            (v, &mut *(w as *mut T))
        })
    }

    /// An iterator visiting all keys in order. The iterator element type is H.
    #[inline]
    pub fn keys<'a>(&'a self) -> impl DoubleEndedIterator<Item = H> + 'a {
        self.handles.iter()
    }

    /// An iterator visiting all entries in order. The iterator element type is &T.
    #[inline]
    pub fn values<'a>(&'a self) -> impl DoubleEndedIterator<Item = &T> + 'a {
        self.handles
            .iter()
            .map(move |v| unsafe { self.entries.get_unchecked(v.index() as usize) })
    }

    /// An iterator visiting all entries in order. The iterator element type is &mut T.
    #[inline]
    pub fn values_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &mut T> + 'a {
        let entries = &mut self.entries;
        self.handles.iter().map(move |v| unsafe {
            let w = entries.get_unchecked_mut(v.index() as usize);
            &mut *(w as *mut T)
        })
    }
}

impl<H: HandleLike, T: Sized> Drop for ObjectPool<H, T> {
    fn drop(&mut self) {
        unsafe {
            for v in &self.handles {
                ::std::ptr::drop_in_place(&mut self.entries[v.index() as usize]);
            }

            self.entries.set_len(0);
        }
    }
}
