extern crate bit_set;

use std::any::Any;
use std::ops::{Index, IndexMut};
use self::bit_set::BitSet;

use super::*;
use super::super::utils::*;

/// The `World` struct are used to manage the whole entity-component system, It keeps
/// tracks of the state of every created `Entity`s. All memthods are supposed to be
/// valid for any context they are available in.
pub struct World {
    entities: HandleSet,
    masks: Vec<BitSet>,
    erasers: Vec<Box<FnMut(&mut Box<Any>, handle::Index) -> ()>>,
    storages: Vec<Option<Box<Any>>>,
}

impl World {
    /// Constructs a new empty `World`.
    pub fn new() -> Self {
        World {
            entities: HandleSet::new(),
            masks: Vec::new(),
            erasers: Vec::new(),
            storages: Vec::new(),
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

    /// Recycles the `Entity` handle, free corresponding components.
    /// and mark its version as dead.
    #[inline]
    pub fn free(&mut self, ent: Entity) -> bool {
        if self.is_alive(ent) {
            for x in self.masks[ent.index() as usize].iter() {
                self.erasers.index_mut(x)(&mut self.storages
                                              .index_mut(x)
                                              .as_mut()
                                              .unwrap(),
                                          ent.index())
            }

            self.masks[ent.index() as usize].clear();
            self.entities.free(ent)
        } else {
            false
        }
    }

    /// Registers a new component type.
    #[inline]
    pub fn register<T>(&mut self)
        where T: Component
    {
        if T::type_index() >= self.storages.len() {
            for _ in self.storages.len()..(T::type_index() + 1) {
                // Keeps downcast type info in closure.
                let eraser = Box::new(|any: &mut Box<Any>, id: handle::Index| {
                    any.downcast_mut::<T::Storage>().unwrap().remove(id);
                });

                self.erasers.push(eraser);
                self.storages.push(None);
            }
        }

        // Returns if we are going to register this component duplicatedly.
        if let Some(_) = self.storages[T::type_index()] {
            return;
        }

        self.storages[T::type_index()] = Some(Box::new(T::Storage::new()));
    }

    /// Add components to entity, returns the reference to component.
    /// If the world did not have component with present `Index`, `None` is returned.
    /// Otherwise the value is updated, and the old value is returned.
    #[inline]
    pub fn assign<T>(&mut self, ent: Entity, value: T) -> Option<T>
        where T: Component
    {
        if self.is_alive(ent) {
            self.masks[ent.index() as usize].insert(T::type_index());
            self._fetch_s_mut::<T>().insert(ent.index(), value)
        } else {
            None
        }
    }

    /// Remove component of entity from the world, returning the component at the `Index`.
    #[inline]
    pub fn remove<T>(&mut self, ent: Entity) -> Option<T>
        where T: Component
    {
        if self.is_alive(ent) {
            self.masks[ent.index() as usize].remove(T::type_index());
            self._fetch_s_mut::<T>().remove(ent.index())
        } else {
            None
        }
    }

    /// Returns true if we have componen in this `Entity`, otherwise false.
    #[inline]
    pub fn has<T>(&self, ent: Entity) -> bool
        where T: Component
    {
        self.entities.is_alive(ent) && self.masks[ent.index() as usize].contains(T::type_index())
    }

    /// Returns a reference to the component corresponding to the `Entity::index`.
    #[inline]
    pub fn fetch<T>(&self, ent: Entity) -> Option<&T>
        where T: Component
    {
        if self.is_alive(ent) {
            self._fetch_s::<T>().get(ent.index())
        } else {
            None
        }
    }

    /// Returns a mutable reference to the component corresponding to the `Entity::index`.
    #[inline]
    pub fn fetch_mut<T>(&mut self, ent: Entity) -> Option<&mut T>
        where T: Component
    {
        if self.is_alive(ent) {
            self._fetch_s_mut::<T>().get_mut(ent.index())
        } else {
            None
        }
    }

    #[inline]
    fn _fetch_s<T>(&self) -> &T::Storage
        where T: Component
    {
        self.storages
            .index(T::type_index())
            .as_ref()
            .expect("Tried to perform an operation on component type that not registered.")
            .downcast_ref::<T::Storage>()
            .unwrap()
    }

    #[inline]
    fn _fetch_s_mut<T>(&mut self) -> &mut T::Storage
        where T: Component
    {
        self.storages
            .index_mut(T::type_index())
            .as_mut()
            .expect("Tried to perform an operation on component type that not registered.")
            .downcast_mut::<T::Storage>()
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut world = World::new();
        assert_eq!(world.size(), 0);

        let e = world.create();
        assert!(world.is_alive(e));
        assert_eq!(world.size(), 1);

        world.free(e);
        assert!(!world.is_alive(e));
        assert_eq!(world.size(), 0);
    }
}
