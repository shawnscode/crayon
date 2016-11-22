extern crate bit_set;

use std::any::{TypeId, Any};
use std::collections::HashMap;
use self::bit_set::BitSet;

use super::*;
use super::super::utils::*;

/// The `World` struct contains all the data, which is entities and
/// their components. All methods are supposed to be valid for any
/// context they are available in.
pub struct World {
    entities: HandleSet,
    masks: Vec<BitSet>,
    storages: HashMap<TypeId, Box<Any>>,
}

impl World {
    /// Constructs a new empty `World`.
    pub fn new() -> Self {
        World {
            entities: HandleSet::new(),
            masks: Vec::new(),
            storages: HashMap::new(),
        }
    }

    /// Creates and returns a unused Entity handle.
    #[inline]
    pub fn create(&mut self) -> Entity {
        let ent = self.entities.create();

        if self.masks.len() <= (ent.index() as usize) {
            self.masks.resize(ent.index() as usize + 1, BitSet::new());
        }

        ent
    }

    /// Returns true if this `Handle` was created by `HandleSet`, and
    /// has not been freed yet.
    #[inline]
    pub fn is_alive(&self, ent: Entity) -> bool {
        self.entities.is_alive(ent)
    }

    /// Returns the number of current alive entities in this `World`.
    #[inline]
    pub fn size(&self) -> usize {
        self.entities.size()
    }

    /// Recycles the `Entity` handle, and mark its version as dead.
    #[inline]
    pub fn free(&mut self, ent: Entity) -> bool {
        self.masks[ent.index() as usize].clear();
        self.entities.free(ent)
    }

    /// Registers a new component type.
    #[inline]
    pub fn register<T>(&mut self)
        where T: Component
    {
        self.storages.insert(TypeId::of::<T>(), Box::new(T::Storage::new()));
    }

    /// Add components to entity, returns the reference to component.
    /// If the world did not have component with present `Index`, `None` is returned.
    /// Otherwise the value is updated, and the old value is returned.
    #[inline]
    pub fn assign<T>(&mut self, ent: Entity, value: T) -> Option<T>
        where T: Component
    {
        self._fetch_s_mut::<T>().insert(ent.index(), value)
    }

    /// Remove component of entity from the world, returning the component at the `Index`.
    #[inline]
    pub fn remove<T>(&mut self, ent: Entity) -> Option<T>
        where T: Component
    {
        if self.entities.is_alive(ent) {
            self._fetch_s_mut::<T>().remove(ent.index())
        } else {
            None
        }
    }

    /// Returns true if we have componen in this `Entity`, otherwise false.
    // #[inline]
    // pub fn has<T>(&self, ent:Entity) -> bool {
    //     if self.entities.is_alive(ent) {
    //         self.masks
    //     }
    //
    //     self.entities.is_alive(ent) && self.masks[ent.index()].contains()
    // }
    /// Returns a reference to the component corresponding to the `Entity::index`.
    #[inline]
    pub fn fetch<T>(&self, ent: Entity) -> Option<&T>
        where T: Component
    {
        self._fetch_s::<T>().get(ent.index())
    }

    /// Returns a mutable reference to the component corresponding to the `Entity::index`.
    #[inline]
    pub fn fetch_mut<T>(&mut self, ent: Entity) -> Option<&mut T>
        where T: Component
    {
        self._fetch_s_mut::<T>().get_mut(ent.index())
    }

    #[inline]
    fn _fetch_s<T>(&self) -> &T::Storage
        where T: Component
    {
        self.storages
            .get(&TypeId::of::<T>())
            .expect("Tried to perform an operation on component type that not registered.")
            .downcast_ref::<T::Storage>()
            .unwrap()
    }

    #[inline]
    fn _fetch_s_mut<T>(&mut self) -> &mut T::Storage
        where T: Component
    {
        self.storages
            .get_mut(&TypeId::of::<T>())
            .expect("Tried to perform an operation on component type that not registered.")
            .downcast_mut::<T::Storage>()
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::component::*;

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct Position {
        x: i32,
        y: i32,
    }

    impl Component for Position {
        type Storage = HashMapStorage<Position>;
    }

    #[test]
    fn world_basic() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.create();
        world.assign::<Position>(e1, Position { x: 1, y: 2 });

        {
            let p = world.fetch::<Position>(e1).unwrap();
            assert_eq!(*p, Position { x: 1, y: 2 });
        }

        world.remove::<Position>(e1);
        assert_eq!(world.fetch::<Position>(e1), None);
    }

    #[test]
    fn world_free() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.create();
        assert!(world.is_alive(e1));
        assert_eq!(world.fetch::<Position>(e1), None);

        world.assign::<Position>(e1, Position { x: 1, y: 2 });
        world.fetch::<Position>(e1).unwrap();

        world.free(e1);
        assert!(!world.is_alive(e1));
        assert_eq!(world.fetch::<Position>(e1), None);
    }
}
