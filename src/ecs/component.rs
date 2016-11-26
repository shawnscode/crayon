use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

use super::super::utils::handle::Index;

lazy_static! {
    /// Lazy initialized id of component. Which produces a continuous index address.
    #[doc(hidden)]
    pub static ref INDEX: AtomicUsize = AtomicUsize::new(0);
}

/// Abstract component trait with associated storage type.
pub trait Component: Any + 'static
    where Self: Sized
{
    type Storage: ComponentStorage<Self> + Any + Send + Sync;

    fn type_index() -> usize;
}

/// Declare a struct as component, and specify the storage strategy. Internally, this
/// macro will impl a internal trait `Component` to provide some useful methods and hints.
#[macro_export]
macro_rules! declare_component {
    ( $CMP:ident, $STORAGE:ident ) => {
        impl $crate::ecs::Component for $CMP {
            type Storage = $STORAGE<$CMP>;

            fn type_index() -> usize {
                use std::sync::atomic::Ordering;
                use $crate::ecs::component::INDEX;
                lazy_static!{static ref ID: usize = INDEX.fetch_add(1, Ordering::SeqCst);};
                *ID
            }
        }
    };
}

/// Traits used to implement a standart/basic storage for components. Choose your
/// components storage layout and strategy by declaring different `ComponentStorage`
/// with corresponding component.
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

/// HashMap based storage which are best suited for rare components.
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

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::Ordering;

    struct Position {}
    struct Direction {}

    declare_component!(Position, HashMapStorage);
    declare_component!(Direction, HashMapStorage);

    #[test]
    fn component_index() {
        assert!(Position::type_index() != Direction::type_index());

        let max = INDEX.load(Ordering::SeqCst);
        assert!(Position::type_index() < max);
        assert!(Direction::type_index() < max);
    }
}