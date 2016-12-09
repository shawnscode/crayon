use std::ptr;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

use bit_set::BitSet;
use super::super::utility::handle::HandleIndex;

lazy_static! {
    /// Lazy initialized id of component. Which produces a continuous index address.
    #[doc(hidden)]
    pub static ref _INDEX: AtomicUsize = AtomicUsize::new(0);
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
                use $crate::ecs::component::_INDEX;
                lazy_static!{static ref ID: usize = _INDEX.fetch_add(1, Ordering::SeqCst);};
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
    /// Returns a reference to the value corresponding to the `HandleIndex`,
    /// without doing bounds checking.
    unsafe fn get(&self, HandleIndex) -> &T;
    /// Returns a mutable reference to the value corresponding to the `HandleIndex`,
    /// without doing bounds checking.
    unsafe fn get_mut(&mut self, HandleIndex) -> &mut T;
    /// Inserts new data for a given `HandleIndex`,
    unsafe fn insert(&mut self, HandleIndex, T);
    /// Removes and returns the data associated with an `HandleIndex`.
    unsafe fn remove(&mut self, HandleIndex) -> T;
}

/// HashMap based storage which are best suited for rare components.
pub struct HashMapStorage<T>
    where T: Component
{
    values: HashMap<HandleIndex, T>,
}

impl<T> ComponentStorage<T> for HashMapStorage<T>
    where T: Component
{
    fn new() -> Self {
        HashMapStorage { values: HashMap::new() }
    }

    unsafe fn get(&self, id: HandleIndex) -> &T {
        self.values.get(&id).unwrap()
    }

    unsafe fn get_mut(&mut self, id: HandleIndex) -> &mut T {
        self.values.get_mut(&id).unwrap()
    }

    unsafe fn insert(&mut self, id: HandleIndex, v: T) {
        self.values.insert(id, v);
    }

    unsafe fn remove(&mut self, id: HandleIndex) -> T {
        self.values.remove(&id).unwrap()
    }
}

/// Vec based storage, supposed to have maximum performance
/// for the components mostly present in entities.
pub struct VecStorage<T>
    where T: Component + ::std::fmt::Debug
{
    mask: BitSet,
    values: Vec<T>,
}

impl<T> ComponentStorage<T> for VecStorage<T>
    where T: Component + ::std::fmt::Debug
{
    fn new() -> Self {
        VecStorage {
            mask: BitSet::new(),
            values: Vec::new(),
        }
    }

    unsafe fn get(&self, id: HandleIndex) -> &T {
        self.values.get_unchecked(id as usize)
    }

    unsafe fn get_mut(&mut self, id: HandleIndex) -> &mut T {
        self.values.get_unchecked_mut(id as usize)
    }

    unsafe fn insert(&mut self, id: HandleIndex, v: T) {
        let len = self.values.len();
        if id as usize >= len {
            self.values.reserve(id as usize + 1 - len);
            self.values.set_len(id as usize + 1);
        }

        // Write the value without reading or dropping
        // the (currently uninitialized) memory.
        self.mask.insert(id as usize);
        ptr::write(self.values.get_unchecked_mut(id as usize), v);
    }

    unsafe fn remove(&mut self, id: HandleIndex) -> T {
        self.mask.remove(id as usize);
        ptr::read(self.get(id))
    }
}

impl<T> Drop for VecStorage<T>
    where T: Component + ::std::fmt::Debug
{
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

        let max = _INDEX.load(Ordering::SeqCst);
        assert!(Position::type_index() < max);
        assert!(Direction::type_index() < max);
    }
}