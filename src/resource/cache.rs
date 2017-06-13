use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::cmp::max;

use utility::hash::HashValue;

#[derive(Debug)]
struct ResourceDesc<T> {
    path: PathBuf,
    hash: HashValue<Path>,
    resource: Arc<T>,
    size: usize,
    next: Option<HashValue<Path>>,
    prev: Option<HashValue<Path>>,
}

/// A cache that holds a limited size of path-resource pairs. When the capacity of
/// the cache is exceeded. The least-recently-used resource, that without any
/// reference outside, will be removed.
pub struct Cache<T> {
    cache: HashMap<HashValue<Path>, ResourceDesc<T>>,
    lru_front: Option<HashValue<Path>>,
    lru_back: Option<HashValue<Path>>,
    used: usize,
    threshold: usize,
    dynamic_threshold: usize,
    extern_ref: usize,
}

impl<T> Cache<T> {
    /// Create a new and empty `Cache`.
    pub fn new(extern_ref: usize, threshold: usize) -> Self {
        Cache {
            cache: HashMap::new(),
            lru_front: None,
            lru_back: None,
            used: 0,
            threshold: threshold,
            dynamic_threshold: threshold,
            extern_ref: extern_ref,
        }
    }

    /// Checks if the cache contains resource associated with given path.
    ///
    /// This operation has no effect on LRU strategy.
    pub fn contains<P>(&self, path: P) -> bool
        where P: AsRef<Path>
    {
        self.cache.get(&path.as_ref().into()).is_some()
    }

    /// Insert a manually created resource and its path into cache.
    ///
    /// If the cache did have this path present, the resource associated with this
    /// path is replaced with new one.
    pub fn insert<P>(&mut self, path: P, size: usize, resource: Arc<T>)
        where P: AsRef<Path>
    {
        let hash = path.as_ref().into();
        let mut desc = ResourceDesc {
            path: path.as_ref().to_owned(),
            hash: hash,
            size: size,
            resource: resource,
            next: None,
            prev: None,
        };

        self.make_room(size);
        self.attach(&mut desc);
        self.cache.insert(hash, desc);
    }

    /// Returns a reference to the value corresponding to the `Path`,
    pub fn get<P>(&mut self, path: P) -> Option<&Arc<T>>
        where P: AsRef<Path>
    {
        let hash = path.as_ref().into();
        if let Some(mut desc) = self.cache.remove(&hash) {
            self.detach(&desc);
            self.attach(&mut desc);
            self.cache.insert(hash, desc);
            Some(&self.cache.get(&hash).unwrap().resource)
        } else {
            None
        }
    }

    fn make_room(&mut self, size: usize) {
        self.used += size;

        if self.used <= self.dynamic_threshold {
            return;
        }

        let mut cursor = self.lru_back;
        while !cursor.is_none() && self.used > self.dynamic_threshold {
            let hash = cursor.unwrap();
            let rc = {
                let desc = self.cache.get(&hash).unwrap();
                cursor = desc.prev;
                Arc::strong_count(&desc.resource)
            };

            // If this resource is referenced by `Cache` only then erase it.
            if rc <= (self.extern_ref + 1) {
                let desc = self.cache.remove(&hash).unwrap();
                self.detach(&desc);
                self.used -= desc.size;
            }
        }

        let delta = self.threshold / 4;
        if self.used > self.dynamic_threshold {
            // If we failed to make spare room for upcoming insertion. Then we
            // simply increase the `dynamic_threshold`.
            self.dynamic_threshold += delta;
        } else {
            self.dynamic_threshold = max(self.dynamic_threshold - delta, self.threshold);
        }
    }

    #[inline]
    fn attach(&mut self, node: &mut ResourceDesc<T>) {
        if let Some(hash) = self.lru_front {
            node.next = Some(hash);
            self.cache.get_mut(&hash).unwrap().prev = Some(node.hash);
        }

        self.lru_front = Some(node.hash);

        if self.lru_back.is_none() {
            self.lru_back = Some(node.hash);
        }
    }

    #[inline]
    fn detach(&mut self, node: &ResourceDesc<T>) {
        if node.prev.is_some() {
            self.cache.get_mut(&node.prev.unwrap()).unwrap().next = node.next;
        }

        if node.next.is_some() {
            self.cache.get_mut(&node.next.unwrap()).unwrap().prev = node.prev;
        }

        if Some(node.hash) == self.lru_front {
            self.lru_front = node.next;
        }

        if Some(node.hash) == self.lru_back {
            self.lru_back = node.prev;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lru() {
        let mut cache = Cache::new(0, 4);
        cache.insert("/1", 2, Arc::new("a1".to_owned()));
        cache.insert("/2", 2, Arc::new("a2".to_owned()));
        assert!(cache.contains("/1"));
        assert!(cache.contains("/2"));

        cache.insert("/3", 2, Arc::new("a3".to_owned()));
        assert!(!cache.contains("/1"));
        assert!(cache.contains("/2"));
        assert!(cache.contains("/3"));

        cache.get("/2").unwrap();
        cache.insert("/4", 2, Arc::new("a4".to_owned()));
        assert!(cache.contains("/2"));
        assert!(!cache.contains("/3"));
        assert!(cache.contains("/4"));
    }

    #[test]
    fn rc() {
        let mut cache = Cache::new(0, 4);

        {
            let a1 = Arc::new("a1".to_owned());
            let a2 = Arc::new("a2".to_owned());
            cache.insert("/1", 2, a1.clone());
            cache.insert("/2", 2, a2.clone());
            assert!(cache.contains("/1"));
            assert!(cache.contains("/2"));

            cache.insert("/3", 2, Arc::new("a3".to_owned()));
            assert!(cache.contains("/1"));
            assert!(cache.contains("/2"));
            assert!(cache.contains("/3"));
        }

        cache.insert("/4", 2, Arc::new("a4".to_owned()));
        assert!(!cache.contains("/1"));
        assert!(!cache.contains("/2"));
        assert!(cache.contains("/3"));
        assert!(cache.contains("/4"));
    }
}