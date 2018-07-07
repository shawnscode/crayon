use std::borrow::Borrow;
use std::collections::HashMap;

use super::location::LocationAtom;
use utils::{Handle, HandlePool};

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
#[derive(Default)]
pub struct Registery<T>
where
    T: Sized + 'static,
{
    handles: HandlePool,
    entries: Vec<Option<Entry<T>>>,
    locations: HashMap<LocationAtom, Handle>,
    recycle: Option<Vec<Handle>>,
}

impl<T> Registery<T>
where
    T: Sized + 'static,
{
    /// Construct a new, empty `Registery`.
    pub fn new() -> Self {
        Registery {
            handles: HandlePool::new(),
            recycle: None,
            entries: Vec::new(),
            locations: HashMap::new(),
        }
    }

    /// Construct a new, empty and passive `Registery`. You have to call `clear` manually
    /// to recycle handles.
    pub fn passive() -> Self {
        Registery {
            handles: HandlePool::new(),
            recycle: Some(Vec::new()),
            entries: Vec::new(),
            locations: HashMap::new(),
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
            self.locations.insert(location, handle.clone());
        }

        handle
    }

    /// Increase the reference count of resource matched `handle`.
    pub fn inc_rc<H>(&mut self, handle: H)
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();

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
    pub fn dec_rc<H>(&mut self, handle: H) -> Option<T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();

        if !self.handles.is_alive(handle) {
            return None;
        }

        unsafe {
            let free = {
                let entry = self.entries
                    .get_unchecked_mut(handle.index() as usize)
                    .as_mut()
                    .unwrap();
                entry.rc -= 1;
                entry.rc == 0
            };

            if free {
                let mut v = None;
                let block = self.entries.get_unchecked_mut(handle.index() as usize);
                ::std::mem::swap(&mut v, block);

                let v = v.unwrap();
                if v.location.is_shared() {
                    self.locations.remove(&v.location);
                }

                let handle = (*handle).into();
                if let Some(recycle) = self.recycle.as_mut() {
                    recycle.push(handle);
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
        if let Some(recycle) = self.recycle.as_mut() {
            for v in recycle.drain(..) {
                self.handles.free(v);
            }
        }
    }

    /// Get the handle with `Location`.
    pub fn lookup<L>(&self, location: L) -> Option<Handle>
    where
        L: Into<LocationAtom>,
    {
        let location = location.into();
        if location.is_shared() {
            self.locations.get(&location).cloned()
        } else {
            None
        }
    }

    /// Get mutable reference to internal value with `Handle`.
    #[inline]
    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
        if self.handles.is_alive(handle) {
            self.entries[handle.index() as usize]
                .as_mut()
                .map(|v| &mut v.value)
        } else {
            None
        }
    }

    /// Get immutable reference to internal value with `Handle`.
    #[inline]
    pub fn get<H>(&self, handle: H) -> Option<&T>
    where
        H: Borrow<Handle>,
    {
        let handle = handle.borrow();
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
    #[inline]
    pub fn is_alive<H>(&self, handle: H) -> bool
    where
        H: Borrow<Handle>,
    {
        self.handles.is_alive(handle)
    }

    /// Get the total number of entries in this `Registery`.
    #[inline]
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Checks if the `Registery` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
