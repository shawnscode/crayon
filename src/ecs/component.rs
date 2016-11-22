use std::any::Any;
use std::collections::HashMap;

use super::super::utils::handle::Index;

/// Abstract component trait with associated storage type.
pub trait Component: Any + 'static
    where Self: Sized
{
    type Storage: ComponentStorage<Self> + Any + Send + Sync;
}

/// Traits used to implement a standart/basic storage for components.
pub trait ComponentStorage<T>
    where T: Component
{
    /// Creates a new `ComponentStorage<T>`. This is called when you register a
    /// new component type within the world.
    fn new() -> Self;
    /// Returns a reference to the value corresponding to the `Index`.
    fn get(&self, Index) -> Option<&T>;
    /// Returns a mutable reference to the value corresponding to the `Index`.
    fn get_mut(&mut self, Index) -> Option<&mut T>;
    /// Inserts new data for a given `Index`.
    /// If the storage did not have this `Index` present, `None` is returned.
    /// Otherwise the value is updated, and the old value is returned.
    fn insert(&mut self, Index, T) -> Option<T>;
    /// Removes and returns the data associated with an `Index`.
    fn remove(&mut self, Index) -> Option<T>;
}

/// HashMap based storage. Best suited for rare components.
pub struct HashMapStorage<T>
    where T: Component
{
    values: HashMap<Index, T>,
}

impl<T> ComponentStorage<T> for HashMapStorage<T>
    where T: Component
{
    fn new() -> Self {
        HashMapStorage { values: HashMap::new() }
    }

    fn get(&self, id: Index) -> Option<&T> {
        self.values.get(&id)
    }

    fn get_mut(&mut self, id: Index) -> Option<&mut T> {
        self.values.get_mut(&id)
    }

    fn insert(&mut self, id: Index, v: T) -> Option<T> {
        self.values.insert(id, v)
    }

    fn remove(&mut self, id: Index) -> Option<T> {
        self.values.remove(&id)
    }
}
