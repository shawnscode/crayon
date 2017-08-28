use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use super::errors::*;
use super::*;

use utility::hash::HashValue;

pub struct ResourceSystemBackend<T>
    where T: Resource + ResourceIndex + 'static
{
    cache: Option<cache::Cache<RwLock<T>>>,
    resources: HashMap<HashValue<Path>, Arc<RwLock<T>>>,
    size: usize,
}

impl<T> ResourceSystemBackend<T>
    where T: Resource + ResourceIndex + 'static
{
    pub fn new() -> Self {
        ResourceSystemBackend {
            cache: None,
            resources: HashMap::new(),
            size: 0,
        }
    }

    /// Register a `Cache<T>` with specified cache capacity.
    pub fn register_cache(&mut self, size: usize) {
        if let Some(mut v) = self.cache.as_mut() {
            v.set_threshold(size);
            return;
        }

        self.cache = Some(cache::Cache::<RwLock<T>>::new(size));
    }

    /// Returns size of all loaded assets from this `ResourceSystemBackend`.
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get<P>(&mut self, path: P) -> Option<ResourceItem<T>>
        where P: AsRef<Path>
    {
        let hash = path.as_ref().into();

        if let Some(rc) = self.resources.get(&hash) {
            if let Some(mut c) = self.cache.as_mut() {
                c.insert(&path, rc.read().unwrap().size(), rc.clone());
            }

            return Some(rc.clone());
        }

        None
    }

    pub fn insert<P>(&mut self, path: P, item: ResourceItem<T>) -> Result<()>
        where P: AsRef<Path>
    {
        let hash = path.as_ref().into();

        if self.resources.contains_key(&hash) {
            bail!("duplicated insert.");
        }

        let size = item.read().unwrap().size();
        if let Some(mut c) = self.cache.as_mut() {
            c.insert(&path, size, item.clone());
        }

        self.resources.insert(hash, item);
        self.size += size;

        Ok(())
    }

    /// Remove internal reference of resources if there is not any external reference exists.
    pub fn unload_unused(&mut self) {
        let mut next = HashMap::new();
        for (k, v) in self.resources.drain() {
            if Arc::strong_count(&v) > 1 {
                next.insert(k, v);
            } else {
                self.size -= v.read().unwrap().size();
            }
        }
        self.resources = next;
    }
}