use std::any::{TypeId, Any};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::collections::HashMap;

use super::*;
use super::component::*;
use super::super::utils::handle::*;

/// The `World` struct contains all the data, which is entities and
/// their components. All methods are supposed to be valid for any
/// context they are available in.
pub struct World {
    entities: RwLock<HandleSet>,
    storages: HashMap<TypeId, Box<Any>>,
}

impl World {
    /// Constructs a new empty `World`.
    pub fn new() -> Self {
        World {
            entities: RwLock::new(HandleSet::new()),
            storages: HashMap::new(),
        }
    }

    /// Creates and returns a unused Entity handle.
    #[inline]
    pub fn create(&mut self) -> Entity {
        self.entities.write().unwrap().create()
    }

    /// Returns true if this `Handle` was created by `HandleSet`, and
    /// has not been freed yet.
    #[inline]
    pub fn is_alive(&self, ent: Entity) -> bool {
        self.entities.read().unwrap().is_alive(ent)
    }

    /// Returns the number of current alive entities in this `World`.
    pub fn size(&self) -> usize {
        self.entities.read().unwrap().size()
    }

    /// Recycles the `Entity` handle, and mark its version as dead.
    #[inline]
    pub fn free(&mut self, ent: Entity) -> bool {
        self.entities.write().unwrap().is_alive(ent)
    }

    /// Registers a new component type.
    pub fn register<T>(&mut self) where T : Component {
        let any = RwLock::new(T::Storage::new());
        self.storages.insert(TypeId::of::<T>(), Box::new(any));
    }

    /// Locks a component's storage for reading.
    #[inline]
    pub fn read<T>(&self) -> RwLockReadGuard<T::Storage> where T : Component {
        self.storages.get(&TypeId::of::<T>())
            .expect("Tried to perform an operation on component type that was not registered.")
            .downcast_ref::<RwLock<T::Storage>>()
            .unwrap()
            .read()
            .unwrap()
    }

    /// Locks a component's storage for writing.
    #[inline]
    pub fn write<T>(&self) -> RwLockWriteGuard<T::Storage> where T : Component {
        self.storages.get(&TypeId::of::<T>())
            .expect("Tried to perform an operation on component type that was not registered.")
            .downcast_ref::<RwLock<T::Storage>>()
            .unwrap()
            .write()
            .unwrap()
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::component::*;

    #[test]
    fn world_basic() {
        let mut world = World::new();
        let h1 = world.create();
        assert!(world.is_alive(h1));
    }

    #[derive(Debug, Copy, Clone)]
    struct Position {
        x: i32,
        y: i32,
    }

    impl Component for Position {
        type Storage = HashMapStorage<Position>;
    }

    impl Position {
        pub fn new(x: i32, y : i32) -> Self {
            Position { x: x, y : y }
        }
    }

    #[test]
    fn world_components() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.create();

        let mut storage = world.write::<Position>();
        unsafe {
            storage.insert(*e1, Position::new(3, 2));
            let pos = storage.get(*e1);
            assert_eq!(pos.x, 3);
            assert_eq!(pos.y, 2);
        }
    }
}
