use std::fmt::Debug;
use std::collections::HashMap;
use std::path::Path;
use std::any::Any;
use std::sync::{Arc, RwLock};

use super::errors::*;
use super::archive;
use super::cache;
use utility::hash::HashValue;

pub trait Resource {
    /// Returns internal memory usages of resource in bytes.
    fn size(&self) -> usize;
}

pub trait ResourceIndex {
    fn index() -> usize;
}

/// `ResourceLoader`
pub trait ResourceLoader: Debug {
    type Item: Resource + ResourceIndex + 'static;

    fn create(file: &mut archive::File) -> Result<Self::Item>;
}

pub struct ResourceSystem {
    archives: archive::ArchiveCollection,
    caches: Vec<Option<Box<Any>>>,
    resources: Vec<Box<Any>>,
}

impl ResourceSystem {
    pub fn new() -> Self {
        ResourceSystem {
            archives: archive::ArchiveCollection::new(),
            caches: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Return mutable reference to `ArchiveCollection`.
    pub fn archives(&mut self) -> &mut archive::ArchiveCollection {
        &mut self.archives
    }

    /// Register a `Cache<T>` with specified cache capacity.
    pub fn register_cache<T>(&mut self, size: usize)
        where T: Resource + ResourceIndex + 'static
    {
        if T::index() >= self.caches.len() {
            for _ in self.caches.len()..(T::index() + 1) {
                self.caches.push(None);
            }
        }

        if let Some(_) = self.caches[T::index()] {
            return;
        }

        self.caches[T::index()] = Some(Box::new(cache::Cache::<RwLock<T>>::new(1, size)));
    }

    pub fn load<T, P>(&mut self, path: P) -> Result<Arc<RwLock<T::Item>>>
        where T: ResourceLoader,
              P: AsRef<Path>
    {
        let hash = path.as_ref().into();
        if let Some(v) = self.table_mut::<T::Item>().get(&hash) {
            return Ok(v.clone());
        }

        if let Some(cache) = self.cache_mut::<T::Item>() {
            if let Some(v) = cache.get(&path) {
                return Ok(v.clone());
            }
        }

        let mut file = self.archives.open(&path.as_ref())?;
        let resource = T::create(file.as_mut())?;
        let size = resource.size();
        let rc = Arc::new(RwLock::new(resource));

        self.table_mut::<T::Item>().insert(hash, rc.clone());

        if let Some(cache) = self.cache_mut::<T::Item>() {
            cache.insert(&path, size, rc.clone());
        }

        Ok(rc)
    }

    /// Remove internal reference of resources if there is not any external reference exists.
    pub fn unload_unused(&mut self) {
        // for i in 0..self.resources.len() {
        //     let rc = if self.caches.get(i).is_none() { 1 } else { 2 };
        //     let mut next = HashMap::new();
        //     for (k, v) in self.resources[i].drain() {
        //         if Arc::strong_count(&v) <= rc {
        //             next.insert(k, v);
        //         }
        //     }

        //     self.resources[i] = next;
        // }
    }

    #[inline]
    fn cache_mut<T>(&mut self) -> Option<&mut cache::Cache<RwLock<T>>>
        where T: Resource + ResourceIndex + 'static
    {
        if let Some(element) = self.caches.get_mut(T::index()) {
            if let Some(ref mut cache) = *element {
                return Some(cache.downcast_mut::<cache::Cache<RwLock<T>>>().unwrap());
            }
        }

        None
    }

    #[inline]
    fn table_mut<T>(&mut self) -> &mut HashMap<HashValue<Path>, Arc<RwLock<T>>>
        where T: Resource + ResourceIndex + 'static
    {
        if T::index() >= self.resources.len() {
            for _ in self.resources.len()..(T::index() + 1) {
                self.resources
                    .push(Box::new(HashMap::<HashValue<Path>, Arc<RwLock<T>>>::new()));
            }
        }

        unsafe {
            self.resources
                .get_unchecked_mut(T::index())
                .downcast_mut()
                .unwrap()
        }
    }
}