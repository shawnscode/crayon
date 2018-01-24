use std::collections::HashMap;

use utils::{Handle, HandlePool};
use super::location::LocationAtom;

struct Entry<T: Sized + 'static> {
    value: T,
    location: LocationAtom,
    rc: usize,
}

impl<T: Sized + 'static> Entry<T> {
    fn new(location: LocationAtom, value: T) -> Self {
        Entry {
            location: location,
            value: value,
            rc: 1,
        }
    }
}

/// Compact resource registery.
pub struct Registery<T>
where
    T: Sized + 'static,
{
    handles: HandlePool,
    delay_free: Vec<Handle>,
    entries: Vec<Option<Entry<T>>>,
    locations: HashMap<LocationAtom, Handle>,
}

impl<T> Registery<T>
where
    T: Sized,
{
    /// Construct a new, empty `Registery`.
    pub fn new() -> Self {
        Registery {
            handles: HandlePool::new(),
            delay_free: Vec::new(),
            entries: Vec::new(),
            locations: HashMap::new(),
        }
    }

    /// Construct a new `Registery` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Registery {
            handles: HandlePool::with_capacity(capacity),
            delay_free: Vec::new(),
            entries: Vec::with_capacity(capacity),
            locations: HashMap::with_capacity(capacity),
        }
    }

    /// Add a new entry to the register.
    pub fn create<L>(&mut self, location: L, value: T) -> Handle
    where
        L: Into<LocationAtom>,
    {
        let location = location.into();
        assert!(self.lookup(location).is_none());

        let handle = self.handles.create();
        let entry = Entry::new(location, value);

        if handle.index() >= self.entries.len() as u32 {
            self.entries.push(Some(entry));
        } else {
            self.entries[handle.index() as usize] = Some(entry);
        }

        if location.is_shared() {
            self.locations.insert(location, handle);
        }

        handle
    }

    /// Increase the reference count of resource matched `handle`.
    pub fn inc_rc(&mut self, handle: Handle) {
        if !self.handles.is_alive(handle) {
            return;
        }

        unsafe {
            self.entries
                .get_unchecked_mut(handle.index() as usize)
                .as_mut()
                .unwrap()
                .rc += 1;
        }
    }

    /// Decrease the reference count of resource matched `handle`. If reference count is zero
    /// after decreasing, it will be deleted from this `Registery`.
    pub fn dec_rc(&mut self, handle: Handle, delay: bool) -> Option<T> {
        if !self.handles.is_alive(handle) {
            return None;
        }

        unsafe {
            let has_reference = {
                let entry = self.entries
                    .get_unchecked_mut(handle.index() as usize)
                    .as_mut()
                    .unwrap();
                entry.rc -= 1;
                entry.rc == 0
            };

            if has_reference {
                let mut v = None;
                let block = self.entries.get_unchecked_mut(handle.index() as usize);
                ::std::mem::swap(&mut v, block);

                let v = v.unwrap();
                self.locations.remove(&v.location);

                if delay {
                    self.delay_free.push(handle);
                } else {
                    self.handles.free(handle);
                }

                return Some(v.value);
            }

            None
        }
    }

    /// Recycles freed handles.
    pub fn clear(&mut self) {
        for v in self.delay_free.drain(..) {
            self.handles.free(v);
        }
    }

    /// Get the handle with `Location`.
    pub fn lookup<L>(&self, location: L) -> Option<Handle>
    where
        L: Into<LocationAtom>,
    {
        let location = location.into();
        if location.is_shared() {
            self.locations.get(&location).map(|v| *v)
        } else {
            None
        }
    }

    /// Get mutable reference to internal value with `Handle`.
    #[inline(always)]
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        if self.handles.is_alive(handle) {
            self.entries[handle.index() as usize]
                .as_mut()
                .map(|v| &mut v.value)
        } else {
            None
        }
    }

    /// Get immutable reference to internal value with `Handle`.
    #[inline(always)]
    pub fn get(&self, handle: Handle) -> Option<&T> {
        if self.handles.is_alive(handle) {
            self.entries[handle.index() as usize]
                .as_ref()
                .map(|v| &v.value)
        } else {
            None
        }
    }

    /// Return true if this `Handle` was created by `Registery`, and has not been
    /// freed yet.
    #[inline(always)]
    pub fn is_alive(&self, handle: Handle) -> bool {
        self.handles.is_alive(handle)
    }

    /// Get the total number of entries in this `Registery`.
    #[inline]
    pub fn len(&self) -> usize {
        self.handles.len()
    }
}
