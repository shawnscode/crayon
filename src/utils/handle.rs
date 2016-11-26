use std::ops::Deref;

/// `Index` type is arbitrary. Keeping it 32-bits allows for
/// a single 64-bits word per `Handle`.
pub type Index = u32;


/// `Handle` is made up of two field, `index` and `version`. `index` are
/// usually used to indicated address into some kind of space. This value
/// is recycled when an `Handle` is freed to save address. However, this
/// means that you could end up with two different `Handle` with identical
/// indices. We solve this by introducing `version`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle {
    index: Index,
    version: Index,
}

impl Handle {
    /// Constructs a new `Handle`.
    #[inline]
    pub fn new(index: Index, version: Index) -> Self {
        Handle {
            index: index,
            version: version,
        }
    }

    /// Constructs a nil/uninitialized `Handle`.
    #[inline]
    pub fn nil() -> Self {
        Handle {
            index: 0,
            version: 0,
        }
    }

    /// Returns true if this `Handle` has been initialized.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.index > 0 || self.version > 0
    }

    /// Invalidate this `Handle` to default value.
    #[inline]
    pub fn invalidate(&mut self) {
        self.index = 0;
        self.version = 0;
    }

    /// Returns index value.
    #[inline]
    pub fn index(&self) -> Index {
        self.index
    }

    /// Returns version value.
    #[inline]
    pub fn version(&self) -> Index {
        self.version
    }
}

impl Deref for Handle {
    type Target = Index;

    fn deref(&self) -> &Index {
        &self.index
    }
}

/// `HandleSet` manages the manipulations of a `Handle` collection, which are
/// created with a continuous `index` field. It also have the ability to find
/// out the current status of a specified `Handle`.
pub struct HandleSet {
    versions: Vec<Index>,
    frees: Vec<Index>,
}

impl HandleSet {
    /// Constructs a new, empty `HandleSet`.
    pub fn new() -> HandleSet {
        HandleSet {
            versions: Vec::new(),
            frees: Vec::new(),
        }
    }

    /// Constructs a new `HandleSet` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> HandleSet {
        let versions = Vec::with_capacity(capacity);
        let mut frees = Vec::with_capacity(capacity);
        for i in 0..versions.len() {
            frees.push(i as Index);
        }

        HandleSet {
            versions: versions,
            frees: frees,
        }
    }

    /// Creates a unused `Handle`.
    pub fn create(&mut self) -> Handle {
        if self.frees.len() > 0 {
            // If we have available free slots.
            let index = self.frees.pop().unwrap() as usize;
            self.versions[index] += 1;
            Handle::new(index as Index, self.versions[index])
        } else {
            // Or we just spawn a new index and corresponding version.
            self.versions.push(1);
            Handle::new(self.versions.len() as Index - 1, 1)
        }
    }

    /// Returns true if this `Handle` was created by `HandleSet`, and has not been
    /// freed yet.
    pub fn is_alive(&self, handle: Handle) -> bool {
        let index = handle.index as usize;
        (index < self.versions.len()) && ((self.versions[index] & 0x1) == 1) &&
        (self.versions[index] == handle.version)
    }

    /// Recycles the `Handle` index, and mark its version as dead.
    pub fn free(&mut self, handle: Handle) -> bool {
        if !self.is_alive(handle) {
            false
        } else {
            self.versions[handle.index as usize] += 1;
            self.frees.push(handle.index);
            true
        }
    }

    /// Returns the total number of alive handle in this `HandleSet`.
    pub fn size(&self) -> usize {
        self.versions.len() - self.frees.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut h2 = Handle::new(2, 4);
        assert_eq!(h2.index, 2);
        assert_eq!(h2.version, 4);
        assert!(h2.is_valid());
        assert_eq!(*h2, 2);

        h2.invalidate();
        assert_eq!(h2.index, 0);
        assert_eq!(h2.version, 0);
        assert!(!h2.is_valid());
        assert_eq!(*h2, 0);
    }

    #[test]
    fn container() {
        use std::collections::HashSet;
        let h1 = Handle::new(1, 1);
        let h2 = Handle::new(1, 2);
        let h3 = Handle::new(2, 2);
        let h4 = Handle::new(1, 1);

        let mut map = HashSet::new();
        assert_eq!(map.insert(h1), true);
        assert_eq!(map.contains(&h1), true);
        assert_eq!(map.insert(h4), false);
        assert_eq!(map.contains(&h4), true);
        assert_eq!(map.insert(h2), true);
        assert_eq!(map.insert(h3), true);
    }

    #[test]
    fn handle_set() {
        let mut set = HandleSet::new();
        assert_eq!(set.size(), 0);

        // Spawn entities.
        let e1 = set.create();
        assert!(e1.is_valid());
        assert!(set.is_alive(e1));
        assert_eq!(set.size(), 1);

        let mut e2 = e1;
        assert!(set.is_alive(e2));
        assert_eq!(set.size(), 1);

        // Invalidate entities.
        e2.invalidate();
        assert!(!e2.is_valid());
        assert!(!set.is_alive(e2));
        assert!(set.is_alive(e1));

        // Free entities.
        let e2 = e1;
        set.free(e2);
        assert!(!set.is_alive(e2));
        assert!(!set.is_alive(e1));
        assert_eq!(set.size(), 0);
    }

    #[test]
    fn index_reuse() {
        let mut set = HandleSet::new();

        assert_eq!(set.size(), 0);

        let mut v = vec![];
        for _ in 0..10 {
            v.push(set.create());
        }

        assert_eq!(set.size(), 10);
        for e in v.iter() {
            set.free(*e);
        }

        for _ in 0..10 {
            let e = set.create();
            assert!((*e as usize) < v.len());
            assert!(v[*e as usize].version() != e.version());
        }
    }
}
