use std::any::{Any};
use std::collections::HashMap;

use super::super::utils::handle::Index;

/// Abstract component trait with associated storage type.
pub trait Component: Any + 'static where Self: Sized {
    type Storage: ComponentStorage<Self> + Any + Send + Sync;
}

pub trait BasicComponentStorage {}

/// Traits used to implement a standart/basic storage for components.
pub trait ComponentStorage<T>: BasicComponentStorage
    where T: Component
{
    /// Creates a new `ComponentStorage<T>`. This is called when you register a
    /// new component type within the world.
    fn new() -> Self;
    /// Clean the storage given a check to figure out if an index
    /// is valid or not. Allows us to safely drop the storage.
    // fn clean<F>(&mut self, F) where F: Fn(Index) -> bool;
    /// Tries reading the data associated with an `Index`.
    unsafe fn get(&self, Index) -> &T;
    /// Tries mutating the data associated with an `Index`.
    unsafe fn get_mut(&mut self, Index) -> &mut T;
    /// Inserts new data for a given `Index`.
    unsafe fn insert(&mut self, Index, T);
    /// Removes the data associated with an `Index.
    unsafe fn remove(&mut self, Index) -> T;
}

/// HashMap based storage. Best suited for rare components.
pub struct HashMapStorage<T>
    where T: Component
{
    values: HashMap<Index, T>,
}

impl<T> BasicComponentStorage for HashMapStorage<T> where T: Component {}

impl<T> ComponentStorage<T> for HashMapStorage<T>
    where T: Component
{
    fn new() -> Self {
        HashMapStorage { values: HashMap::new() }
    }

    unsafe fn get(&self, id: Index) -> &T {
        self.values.get(&id).unwrap()
    }

    unsafe fn get_mut(&mut self, id: Index) -> &mut T {
        self.values.get_mut(&id).unwrap()
    }

    unsafe fn insert(&mut self, id: Index, v: T) {
        self.values.insert(id, v);
    }

    unsafe fn remove(&mut self, id: Index) -> T {
        self.values.remove(&id).unwrap()
    }
}
