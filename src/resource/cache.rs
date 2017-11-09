use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use utils::hash::HashValue;

/// A slim proxy trait that adds a standardized interface of meansurable resources.
pub trait Meansurable: Send + Sync + 'static {
    fn size(&self) -> usize;
}

#[derive(Debug)]
struct ResourceDesc<T> {
    hash: HashValue<Path>,
    resource: Arc<T>,
    size: usize,
    next: Option<HashValue<Path>>,
    prev: Option<HashValue<Path>>,
}

/// A cache that holds a limited size of path-resource pairs. When the capacity of
/// the cache is exceeded, the least-recently-used resource will be removed.
pub struct Cache<T>
    where T: Meansurable
{
    cache: HashMap<HashValue<Path>, ResourceDesc<T>>,
    lru_front: Option<HashValue<Path>>,
    lru_back: Option<HashValue<Path>>,
    used: usize,
    threshold: usize,
}

impl<T> Cache<T>
    where T: Meansurable
{
    /// Create a new and empty `Cache`.
    pub fn new(threshold: usize) -> Self {
        Cache {
            cache: HashMap::new(),
            lru_front: None,
            lru_back: None,
            used: 0,
            threshold: threshold,
        }
    }

    /// Reset the threshold of this cache.
    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold;
        self.make_room(0);
    }

    /// Checks if the cache contains resource associated with given path.
    ///
    /// This operation has no effect on LRU strategy.
    pub fn contains<P>(&self, path: P) -> bool
        where P: Into<HashValue<Path>>
    {
        self.cache.get(&path.into()).is_some()
    }

    /// Insert a manually created resource and its path into cache.
    ///
    /// If the cache did have this path present, the resource associated with this
    /// path is replaced with new one.
    pub fn insert<P>(&mut self, path: P, resource: Arc<T>)
        where P: Into<HashValue<Path>>
    {
        let hash = path.into();
        let size = resource.size();

        if let Some(desc) = self.cache.remove(&hash) {
            self.detach(&desc);
            self.used -= desc.size;
        }

        let mut desc = ResourceDesc {
            hash: hash,
            size: size,
            resource: resource,
            next: None,
            prev: None,
        };

        if self.make_room(size) {
            self.attach(&mut desc);
            self.cache.insert(hash, desc);
        }
    }

    /// Returns a reference to the value corresponding to the `Path`,
    pub fn get<P>(&mut self, path: P) -> Option<&Arc<T>>
        where P: Into<HashValue<Path>>
    {
        let hash = path.into();
        if let Some(mut desc) = self.cache.remove(&hash) {
            self.detach(&desc);
            self.attach(&mut desc);
            self.cache.insert(hash, desc);
            Some(&self.cache.get(&hash).unwrap().resource)
        } else {
            None
        }
    }

    fn make_room(&mut self, size: usize) -> bool {
        if (self.used + size) <= self.threshold {
            self.used += size;
            return true;
        }

        let mut cursor = self.lru_back;
        while !cursor.is_none() {
            let hash = cursor.unwrap();
            let desc = self.cache.remove(&hash).unwrap();
            self.detach(&desc);
            self.used -= desc.size;
            cursor = desc.prev;

            if (self.used + size) <= self.threshold {
                self.used += size;
                return true;
            }
        }

        false
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

    impl Meansurable for String {
        fn size(&self) -> usize {
            self.len()
        }
    }

    #[test]
    fn lru() {
        let mut cache = Cache::new(4);
        cache.insert("/1", Arc::new("a1".to_owned()));
        cache.insert("/2", Arc::new("a2".to_owned()));
        cache.insert("/2", Arc::new("a2".to_owned()));
        assert!(cache.contains("/1"));
        assert!(cache.contains("/2"));

        cache.insert("/3", Arc::new("a3".to_owned()));
        assert!(!cache.contains("/1"));
        assert!(cache.contains("/2"));
        assert!(cache.contains("/3"));

        cache.get("/2").unwrap();
        cache.insert("/4", Arc::new("a4".to_owned()));
        assert!(cache.contains("/2"));
        assert!(!cache.contains("/3"));
        assert!(cache.contains("/4"));
    }

    #[test]
    fn rc() {
        let mut cache = Cache::new(4);

        {
            let a1 = Arc::new("a1".to_owned());
            let a2 = Arc::new("a2".to_owned());
            cache.insert("/1", a1.clone());
            cache.insert("/2", a2.clone());
            assert!(cache.contains("/1"));
            assert!(cache.contains("/2"));

            cache.insert("/3", Arc::new("a3".to_owned()));
            assert!(!cache.contains("/1"));
            assert!(cache.contains("/2"));
            assert!(cache.contains("/3"));
        }

        cache.insert("/4", Arc::new("a4".to_owned()));
        assert!(!cache.contains("/1"));
        assert!(!cache.contains("/2"));
        assert!(cache.contains("/3"));
        assert!(cache.contains("/4"));
    }

    #[test]
    fn zero_size() {
        let mut cache = Cache::new(0);

        cache.insert("/1", Arc::new("a4".to_owned()));
        assert!(!cache.contains("/1"));
    }

    #[test]
    fn reset_threshold() {
        let mut cache = Cache::new(4);
        cache.insert("/1", Arc::new("a1".to_owned()));
        cache.insert("/2", Arc::new("a2".to_owned()));

        cache.set_threshold(7);
        cache.insert("/3", Arc::new("a3".to_owned()));
        cache.insert("/4", Arc::new("a4".to_owned()));
        assert!(!cache.contains("/1"));
        assert!(cache.contains("/2"));
        assert!(cache.contains("/3"));
        assert!(cache.contains("/4"));

        cache.set_threshold(4);
        assert!(!cache.contains("/0"));
        assert!(!cache.contains("/2"));
        assert!(cache.contains("/3"));
        assert!(cache.contains("/4"));
    }
}