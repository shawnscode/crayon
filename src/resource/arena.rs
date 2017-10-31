use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use utils::HashValue;
use super::Resource;
use super::cache::Cache;

/// The context type which manages resource of one type. It also provides internal
/// cache mechanism based on LRU strategy.
pub struct ArenaWithCache<T>
    where T: Resource
{
    cache: Cache<T>,
    resources: HashMap<HashValue<Path>, Arc<T>>,
}

impl<T> ArenaWithCache<T>
    where T: Resource
{
    /// Create a new resource arena with the specified cache capactiy.
    pub fn with_capacity(size: usize) -> Self {
        let cache = Cache::<T>::new(size);

        ArenaWithCache {
            cache: cache,
            resources: HashMap::new(),
        }
    }

    /// Reset the internal cache size.
    #[inline]
    pub fn set_cache_size(&mut self, size: usize) {
        self.cache.set_threshold(size);
    }

    /// Returns a clone of the resource corresponding to the key.
    pub fn get<H>(&mut self, hash: H) -> Option<Arc<T>>
        where H: Into<HashValue<Path>>
    {
        let hash = hash.into();
        if let Some(rc) = self.resources.get(&hash) {
            self.cache.insert(hash, rc.size(), rc.clone());
            return Some(rc.clone());
        }

        None
    }

    /// Inserts a key-value pair into the arena.
    ///
    /// If the arena did not have this key present, `None` is returned. Otherwise the
    /// old value is returned.
    pub fn insert<H>(&mut self, hash: H, item: Arc<T>) -> Option<Arc<T>>
        where H: Into<HashValue<Path>>
    {
        let hash = hash.into();
        let size = item.size();
        self.cache.insert(hash, size, item.clone());
        self.resources.insert(hash, item)
    }

    /// Iterates the arena, removing all resources that have no external references.
    pub fn unload_unused(&mut self) {
        let mut next = HashMap::new();
        for (k, v) in self.resources.drain() {
            if Arc::strong_count(&v) > 1 {
                next.insert(k, v);
            }
        }
        self.resources = next;
    }
}