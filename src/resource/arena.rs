use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use super::cache::{Cache, Meansurable};
use utils::HashValue;

pub struct Arena<T>
    where T: Sized
{
    resources: HashMap<HashValue<Path>, Arc<T>>,
}

impl<T> Arena<T>
    where T: Sized
{
    /// Create a new resource arena.
    pub fn new() -> Self {
        Arena { resources: HashMap::new() }
    }

    /// Returns a clone of the resource corresponding to the key.
    pub fn get<H>(&self, hash: H) -> Option<&Arc<T>>
        where H: Into<HashValue<Path>>
    {
        self.resources.get(&hash.into())
    }

    /// Inserts a key-value pair into the arena.
    ///
    /// If the arena did not have this key present, `None` is returned. Otherwise the
    /// old value is returned.
    pub fn insert<H>(&mut self, hash: H, item: Arc<T>) -> Option<Arc<T>>
        where H: Into<HashValue<Path>>
    {
        self.resources.insert(hash.into(), item)
    }

    /// Iterates the arena, removing all resources that have no external references.
    pub fn unload_unused(&mut self, closure: Option<&Fn(HashValue<Path>, Arc<T>)>) {
        let mut next = HashMap::new();
        for (k, v) in self.resources.drain() {
            if Arc::strong_count(&v) > 1 {
                next.insert(k, v);
            } else {
                if let Some(ref closure) = closure {
                    closure(k, v);
                }
            }
        }
        self.resources = next;
    }
}

pub struct ArenaWithCache<T>
    where T: Meansurable + Sized
{
    cache: Cache<T>,
    resources: HashMap<HashValue<Path>, Arc<T>>,
}

impl<T> ArenaWithCache<T>
    where T: Meansurable + Sized
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
    pub fn get<H>(&mut self, hash: H) -> Option<&Arc<T>>
        where H: Into<HashValue<Path>>
    {
        let hash = hash.into();
        if let Some(rc) = self.resources.get(&hash) {
            self.cache.insert(hash, rc.clone());
            return Some(rc);
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
        self.cache.insert(hash, item.clone());
        self.resources.insert(hash, item)
    }

    /// Iterates the arena, removing all resources that have no external references.
    pub fn unload_unused(&mut self, closure: Option<&Fn(HashValue<Path>, Arc<T>)>) {
        let mut next = HashMap::new();
        for (k, v) in self.resources.drain() {
            if Arc::strong_count(&v) > 1 {
                next.insert(k, v);
            } else {
                if let Some(ref closure) = closure {
                    closure(k, v);
                }
            }
        }
        self.resources = next;
    }
}
