use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use utils::HashValue;
use super::{Ptr, Resource};
use super::cache::Cache;

/// The context type which manages resource of one type. It also provides internal
/// cache mechanism based on LRU strategy.
pub struct ArenaWithCache<T>
    where T: Resource
{
    cache: Cache<RwLock<T>>,
    resources: HashMap<HashValue<Path>, Arc<RwLock<T>>>,
    info: ArenaInfo,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ArenaInfo {
    pub size: usize,
    pub num: usize,
}

impl<T> ArenaWithCache<T>
    where T: Resource
{
    /// Create a new resource arena with the specified cache capactiy.
    pub fn with_capacity(size: usize) -> Self {
        let cache = Cache::<RwLock<T>>::new(size);

        ArenaWithCache {
            cache: cache,
            resources: HashMap::new(),
            info: ArenaInfo::default(),
        }
    }

    #[inline]
    pub fn info(&self) -> ArenaInfo {
        self.info
    }

    /// Reset the internal cache size.
    #[inline]
    pub fn set_cache_size(&mut self, size: usize) {
        self.cache.set_threshold(size);
    }

    /// Returns a clone of `Ptr` to the resource corresponding to the path.
    pub fn get(&mut self, hash: HashValue<Path>) -> Option<Ptr<T>> {
        if let Some(rc) = self.resources.get(&hash) {
            let size = rc.read().unwrap().size();
            self.cache.insert(hash, size, rc.clone());
            return Some(rc.clone());
        }

        None
    }

    /// Inserts a resource into arena.
    pub fn insert(&mut self, hash: HashValue<Path>, item: Ptr<T>) -> Option<Ptr<T>> {
        let size = item.read().unwrap().size();

        self.info.size += size;
        self.info.num += 1;
        self.cache.insert(hash, size, item.clone());

        let old = self.resources.insert(hash, item);
        if let Some(ref v) = old {
            self.info.size -= v.read().unwrap().size();
            self.info.num -= 1;
        }

        old
    }

    /// Removes unused resources.
    pub fn unload_unused(&mut self) {
        let mut next = HashMap::new();
        for (k, v) in self.resources.drain() {
            if Arc::strong_count(&v) > 1 {
                next.insert(k, v);
            } else {
                self.info.size -= v.read().unwrap().size();
                self.info.num -= 1;
            }
        }

        self.resources = next;
    }
}