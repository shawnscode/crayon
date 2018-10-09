use crayon::utils::{FastHashMap, HandleLike};

pub struct Component<H: HandleLike, T> {
    pub remap: FastHashMap<H, usize>,
    pub handles: Vec<H>,
    pub data: Vec<T>,
}

impl<H: HandleLike, T> Component<H, T> {
    /// Creates a new and empty storage for components.
    #[inline]
    pub fn new() -> Self {
        Component {
            remap: FastHashMap::default(),
            handles: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Adds a value to this storage, replacing the existing value, if any, that
    /// is equal to the given one. Returns the replaced value.
    pub fn add(&mut self, handle: H, mut v: T) -> Option<T> {
        if let Some(&index) = self.remap.get(&handle) {
            unsafe {
                ::std::ptr::swap(&mut self.data[index], &mut v);
                Some(v)
            }
        } else {
            self.remap.insert(handle, self.data.len());
            self.handles.push(handle);
            self.data.push(v);
            None
        }
    }

    /// Returns true if the storage contains a value.
    #[inline]
    pub fn contains(&self, handle: H) -> bool {
        self.remap.contains_key(&handle)
    }

    /// Removes and returns the value in the storage, if any, that is equal to
    /// the given one.
    pub fn remove(&mut self, handle: H) -> Option<T> {
        if let Some(index) = self.remap.remove(&handle) {
            if self.remap.len() != index {
                *self.remap.get_mut(&self.handles[index]).unwrap() = index;
            }

            self.handles.swap_remove(index);

            Some(self.data.swap_remove(index))
        } else {
            None
        }
    }

    /// Returns a reference to the value in the storage, if any, that is equal
    /// to the given value.
    #[inline]
    pub fn get(&self, handle: H) -> Option<&T> {
        let data = &self.data;
        self.remap.get(&handle).map(|&index| &data[index])
    }

    /// Returns a mutable reference to the value in the storage, if any, that is
    /// equal to the given value.
    #[inline]
    pub fn get_mut(&mut self, handle: H) -> Option<&mut T> {
        let data = &mut self.data;
        self.remap.get(&handle).map(move |&index| &mut data[index])
    }
}
