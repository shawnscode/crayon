//!

use std::collections::HashSet;
use std::path::Path;
use std::borrow::Borrow;
use std::any::Any;
use std::sync::{Arc, RwLock};

use utils::HashValue;
use super::{Ptr, Resource, ResourceIndex, ResourceParser};
use super::arena::ArenaWithCache;
use super::filesystem::{Filesystem, FilesystemDriver};
use super::errors::*;

pub struct ResourceSystem {
    filesystem: FilesystemDriver,
    arenas: Vec<Option<Box<Any>>>,
    locks: HashSet<HashValue<Path>>,
}

impl ResourceSystem {
    pub fn new() -> Result<Self> {
        Ok(ResourceSystem {
               filesystem: FilesystemDriver::new(),
               arenas: Vec::new(),
               locks: HashSet::new(),
           })
    }

    /// Register a new resource type.
    #[inline]
    pub fn register<T>(&mut self)
        where T: Resource + ResourceIndex + 'static
    {
        let index = T::type_index();
        if index >= self.arenas.len() {
            for _ in self.arenas.len()..(index + 1) {
                self.arenas.push(None)
            }
        }

        // Returns if we are going to register this resource duplicatedly.
        if self.arenas[index].is_some() {
            return;
        }

        self.arenas[index] = Some(Box::new(ArenaWithCache::<T>::with_capacity(0)));
    }

    /// Mount a file-system drive with identifier.
    #[inline]
    pub fn mount<S, F>(&mut self, ident: S, fs: F) -> Result<()>
        where S: Borrow<str>,
              F: Filesystem + 'static
    {
        self.filesystem.mount(ident, fs)
    }

    /// Unmount a file-system from this collection.
    #[inline]
    pub fn unmount<S>(&mut self, ident: S)
        where S: Borrow<str>
    {
        self.filesystem.unmount(ident);
    }

    /// Return whether the path points at an existing file.
    #[inline]
    pub fn exists<P>(&self, path: P) -> bool
        where P: AsRef<Path>
    {
        self.filesystem.exists(path)
    }

    /// Read all bytes until EOF in this source, and parse it with specified
    /// `ResourceParser`.
    pub fn load<T, P>(&mut self, path: P) -> Result<Ptr<T::Item>>
        where T: ResourceParser,
              P: AsRef<Path>
    {
        let hash = path.as_ref().into();
        if let Some(rc) = self.arena_mut::<T::Item>().get(hash) {
            return Ok(rc);
        }

        if self.locks.contains(&hash) {
            bail!(ErrorKind::CircularReferenceFound);
        }

        let rc = {
            self.locks.insert(hash);
            let bytes = self.filesystem.load(path.as_ref())?;
            let resource = T::parse(bytes)?;
            self.locks.remove(&hash);

            Arc::new(RwLock::new(resource))
        };

        self.arena_mut::<T::Item>().insert(hash, rc.clone());
        Ok(rc)
    }

    /// Unload unused, there is no external references, resources from memory.
    pub fn unload_unused(&mut self) {
        for v in &mut self.arenas {
            if let &mut Some(ref mut arena) = v {
                arena
                    .downcast_mut::<Box<Arena>>()
                    .unwrap()
                    .unload_unused_internal();
            }
        }
    }

    fn arena_mut<T>(&mut self) -> &mut Box<ArenaWithCache<T>>
        where T: Resource + ResourceIndex + 'static
    {
        self.arenas
            .get_mut(T::type_index())
            .expect("Tried to perform an operation on resource type that not registered.")
            .as_mut()
            .expect("Tried to perform an operation on resource type that not registered.")
            .downcast_mut::<Box<ArenaWithCache<T>>>()
            .unwrap()
    }
}

/// Anonymous operations helper.
trait Arena {
    fn unload_unused_internal(&mut self);
}

impl<T> Arena for ArenaWithCache<T>
    where T: Resource + ResourceIndex
{
    fn unload_unused_internal(&mut self) {
        self.unload_unused()
    }
}