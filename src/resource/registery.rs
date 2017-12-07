use std::collections::HashMap;

use utils::{HandlePool, Handle};
use super::location::{Location, LocationAtom};

struct Entry<T, Label>
    where T: Sized + 'static,
          Label: Sized + PartialEq + Eq + 'static
{
    value: T,
    label: Label,
    location: LocationAtom,
}

/// Compact resource registery.
pub struct Registery<T, Label>
    where T: Sized + 'static,
          Label: Sized + PartialEq + Eq + 'static
{
    handles: HandlePool,
    entries: Vec<Option<Entry<T, Label>>>,
    locations: HashMap<LocationAtom, Handle>,
}

impl<T, Label> Registery<T, Label>
    where T: Sized,
          Label: Sized + PartialEq + Eq
{
    /// Construct a new, empty `Registery`.
    pub fn new() -> Self {
        Registery {
            handles: HandlePool::new(),
            entries: Vec::new(),
            locations: HashMap::new(),
        }
    }

    /// Construct a new `Registery` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Registery {
            handles: HandlePool::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
            locations: HashMap::with_capacity(capacity),
        }
    }

    /// Add a new entry to the register.
    pub fn add(&mut self, location: Location, label: Label, value: T) -> Handle {
        let hash = location.hash();
        let entry = Entry {
            location: hash,
            label: label,
            value: value,
        };

        let handle = self.handles.create();

        if handle.index() >= self.entries.len() as u32 {
            self.entries.push(Some(entry));
        } else {
            self.entries[handle.index() as usize] = Some(entry);
        }

        if location.is_shared() {
            self.locations.insert(hash, handle);
        }

        handle
    }

    /// Get the handle with `Location`.
    pub fn lookup(&self, location: Location) -> Option<Handle> {
        let hash = location.hash();
        if hash.is_shared() {
            self.locations.get(&hash).map(|v| *v)
        } else {
            None
        }
    }

    /// Remove all entries matching with `label` incrementally.
    pub fn delete(&mut self, label: Label) -> DeleteIter<T, Label> {
        DeleteIter {
            index: 0,
            label: label,
            registery: self,
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

pub struct DeleteIter<'a, T, Label>
    where T: Sized + 'static,
          Label: Sized + PartialEq + Eq + 'static
{
    index: usize,
    label: Label,
    registery: &'a mut Registery<T, Label>,
}

impl<'a, T, Label> Iterator for DeleteIter<'a, T, Label>
    where T: Sized + 'static,
          Label: Sized + PartialEq + Eq + 'static
{
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            for i in self.index..self.registery.entries.len() {
                let v = self.registery.entries.get_unchecked_mut(i);

                let location = if let &mut Some(ref entry) = v {
                    if entry.label == self.label {
                        Some(entry.location)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(location) = location {
                    *v = None;

                    if location.is_shared() {
                        self.registery.locations.remove(&location).unwrap();
                    }

                    return Some(self.registery.handles.free_at(i).unwrap());
                }
            }

            None
        }
    }
}
