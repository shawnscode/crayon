use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use super::errors::*;
use super::archive;
use super::cache;
use super::{Resource, ResourceIndex, ResourceLoader};

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

    /// Load resource from archive collections at specified path. It could
    /// be returned from internal cache or load from disk directly.
    pub fn load<L, P>(&mut self,
                      archives: &archive::ArchiveCollection,
                      path: P)
                      -> Result<Arc<RwLock<T>>>
        where L: ResourceLoader<Item = T>,
              P: AsRef<Path>
    {
        let hash = path.as_ref().into();

        if let Some(rc) = self.resources.get(&hash) {
            if let Some(mut c) = self.cache.as_mut() {
                c.insert(&path, rc.read().unwrap().size(), rc.clone());
            }
            return Ok(rc.clone());
        }

        let mut file = archives.open(&path.as_ref())?;
        let resource = L::load_from_file(file.as_mut())?;
        let size = resource.size();
        let rc = Arc::new(RwLock::new(resource));

        self.resources.insert(hash, rc.clone());
        self.size += size;

        if let Some(mut c) = self.cache.as_mut() {
            c.insert(&path, size, rc.clone());
        }

        Ok(rc)
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