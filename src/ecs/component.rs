//! Abstract `Component` trait and some common storage stratedy.

use std::collections::HashMap;
use std::ptr;

use ecs::bitset::DynamicBitSet;
use utils::handle::HandleIndex;

/// Abstract component trait with associated storage type.
pub trait Component: Send + Sync + Sized + 'static {
    type Arena: Arena<Component = Self>;
}

/// Traits used to implement a standart/basic storage for components. Choose your
/// components storage layout and strategy by declaring different `Arena`
/// with corresponding component.
pub trait Arena: Send + Sync {
    type Component;

    /// Gets a reference to the value corresponding to the `HandleIndex`,
    fn get(&self, HandleIndex) -> Option<&Self::Component>;

    /// Gets a mutable reference to the value at the `HandleIndex`, without doing
    /// bounds checking.
    unsafe fn get_unchecked(&self, HandleIndex) -> &Self::Component;

    /// Gets a mutable reference to the value corresponding to the `HandleIndex`,
    fn get_mut(&mut self, HandleIndex) -> Option<&mut Self::Component>;

    /// Gets a mutable reference to the value at the `HandleIndex`, without doing
    /// bounds checking.
    unsafe fn get_unchecked_mut(&mut self, HandleIndex) -> &mut Self::Component;

    /// Inserts new data for a given `HandleIndex`,
    fn insert(&mut self, HandleIndex, Self::Component) -> Option<Self::Component>;

    /// Removes and returns the data associated with an `HandleIndex`.
    fn remove(&mut self, HandleIndex) -> Option<Self::Component>;
}

/// `HashMap` based storage which are best suited for rare components.
pub struct HashMapArena<T: Component> {
    values: HashMap<HandleIndex, T>,
}

impl<T: Component> Default for HashMapArena<T> {
    fn default() -> Self {
        HashMapArena {
            values: HashMap::new(),
        }
    }
}

impl<T> Arena for HashMapArena<T>
where
    T: Component<Arena = Self>,
{
    type Component = T;

    #[inline]
    fn get(&self, id: HandleIndex) -> Option<&T> {
        self.values.get(&id)
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: HandleIndex) -> &T {
        &self.values[&id]
    }

    #[inline]
    fn get_mut(&mut self, id: HandleIndex) -> Option<&mut T> {
        self.values.get_mut(&id)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, id: HandleIndex) -> &mut T {
        self.values.get_mut(&id).unwrap()
    }

    #[inline]
    fn insert(&mut self, id: HandleIndex, v: T) -> Option<T> {
        self.values.insert(id, v)
    }

    #[inline]
    fn remove(&mut self, id: HandleIndex) -> Option<T> {
        self.values.remove(&id)
    }
}

/// Vec based storage, supposed to have maximum performance for the components
/// mostly present in entities.
pub struct VecArena<T: Component> {
    mask: DynamicBitSet,
    values: Vec<T>,
}

impl<T: Component> Default for VecArena<T> {
    fn default() -> Self {
        VecArena {
            mask: DynamicBitSet::new(),
            values: Vec::new(),
        }
    }
}

impl<T> Arena for VecArena<T>
where
    T: Component<Arena = Self>,
{
    type Component = T;

    #[inline]
    fn get(&self, id: HandleIndex) -> Option<&T> {
        if self.mask.contains(id as usize) {
            self.values.get(id as usize)
        } else {
            None
        }
    }

    #[inline]
    unsafe fn get_unchecked(&self, id: HandleIndex) -> &T {
        self.values.get_unchecked(id as usize)
    }

    #[inline]
    fn get_mut(&mut self, id: HandleIndex) -> Option<&mut T> {
        if self.mask.contains(id as usize) {
            self.values.get_mut(id as usize)
        } else {
            None
        }
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, id: HandleIndex) -> &mut T {
        self.values.get_unchecked_mut(id as usize)
    }

    fn insert(&mut self, id: HandleIndex, v: T) -> Option<T> {
        unsafe {
            let len = self.values.len();
            if id as usize >= len {
                self.values.reserve(id as usize + 1 - len);
                self.values.set_len(id as usize + 1);
            }

            // Write the value without reading or dropping
            // the (currently uninitialized) memory.
            let value = if self.mask.contains(id as usize) {
                Some(ptr::read(self.get_unchecked(id)))
            } else {
                self.mask.insert(id as usize);
                None
            };

            ptr::write(self.values.get_unchecked_mut(id as usize), v);
            value
        }
    }

    fn remove(&mut self, id: HandleIndex) -> Option<T> {
        unsafe {
            if self.mask.contains(id as usize) {
                self.mask.remove(id as usize);
                Some(ptr::read(self.get_unchecked(id)))
            } else {
                None
            }
        }
    }
}

impl<T: Component> Drop for VecArena<T> {
    fn drop(&mut self) {
        unsafe {
            for i in self.mask.iter() {
                ptr::read(self.values.get_unchecked(i));
            }

            self.values.set_len(0);
            self.mask.clear();
        }
    }
}
