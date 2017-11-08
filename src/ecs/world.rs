//! The `World` struct contains entities and its the component arenas.

use std::any::Any;
use std::borrow::Borrow;
use std::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use bit_set::BitSet;

use super::*;
use super::super::utils::{HandleIndex, HandlePool, HandleIter};

/// The `World` struct are used to manage the whole entity-component system, It keeps
/// tracks of the state of every created `Entity`s. All memthods are supposed to be
/// valid for any context they are available in.
pub struct World {
    entities: HandlePool,
    masks: Vec<BitSet>,
    erasers: Vec<Box<FnMut(&Any, HandleIndex) -> () + Send + Sync>>,
    arenas: Vec<Option<Box<Any + Send + Sync>>>,
}

/// Make sure that `World` can be used on multi-threads.
unsafe impl Send for World {}
unsafe impl Sync for World {}

impl World {
    /// Constructs a new empty `World`.
    pub fn new() -> Self {
        World {
            entities: HandlePool::new(),
            masks: Vec::new(),
            erasers: Vec::new(),
            arenas: Vec::new(),
        }
    }

    /// Creates and returns a unused Entity handle.
    pub fn create(&mut self) -> Entity {
        let ent = self.entities.create();

        if self.masks.len() <= (ent.index() as usize) {
            self.masks.resize(ent.index() as usize + 1, BitSet::new());
        }

        ent
    }

    /// Create a entity builder.
    pub fn build(&mut self) -> EntityBuilder {
        EntityBuilder {
            entity: self.create(),
            world: self,
        }
    }

    /// Returns true if this `Handle` was created by `HandlePool`, and
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
    pub fn free(&mut self, ent: Entity) -> bool {
        if self.is_alive(ent) {
            for x in self.masks[ent.index() as usize].iter() {
                let erase = &mut self.erasers[x];
                erase(self.arenas[x].as_ref().unwrap().as_ref(), ent.index())
            }

            self.masks[ent.index() as usize].clear();
            self.entities.free(ent)
        } else {
            false
        }
    }

    /// Registers a new component type.
    pub fn register<T>(&mut self)
        where T: Component
    {
        if T::type_index() >= self.arenas.len() {
            for _ in self.arenas.len()..(T::type_index() + 1) {
                // Keeps downcast type info in closure.
                let eraser = Box::new(|any: &Any, id: HandleIndex| {
                                          any.downcast_ref::<RwLock<T::Storage>>()
                                              .unwrap()
                                              .write()
                                              .unwrap()
                                              .remove(id);
                                      });

                self.erasers.push(eraser);
                self.arenas.push(None);
            }
        }

        // Returns if we are going to register this component duplicatedly.
        if let Some(_) = self.arenas[T::type_index()] {
            return;
        }

        self.arenas[T::type_index()] = Some(Box::new(RwLock::new(T::Storage::new())));
    }

    /// Add components to entity, returns the old value if exists.
    pub fn add<T>(&mut self, ent: Entity, value: T) -> Option<T>
        where T: Component
    {
        if self.is_alive(ent) {
            let result = if self.masks[ent.index() as usize].contains(T::type_index()) {
                self.raw_arena::<T>().write().unwrap().remove(ent.index())
            } else {
                self.masks[ent.index() as usize].insert(T::type_index());
                None
            };

            self.raw_arena::<T>()
                .write()
                .unwrap()
                .insert(ent.index(), value);
            result
        } else {
            None
        }
    }

    pub fn add_with_default<T>(&mut self, ent: Entity) -> Option<T>
        where T: Component + Default
    {
        self.add(ent, Default::default())
    }

    /// Remove component of entity from the world, returning the component at the `HandleIndex`.
    pub fn remove<T>(&mut self, ent: Entity) -> Option<T>
        where T: Component
    {
        if self.masks[ent.index() as usize].contains(T::type_index()) {
            self.masks[ent.index() as usize].remove(T::type_index());
            self.raw_arena::<T>().write().unwrap().remove(ent.index())
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

    /// Returns a reference to the component corresponding to the `Entity`.
    pub fn get<T>(&self, ent: Entity) -> Option<T>
        where T: Component + Copy
    {
        if self.has::<T>(ent) {
            unsafe {
                Some(*self.raw_arena::<T>()
                          .read()
                          .unwrap()
                          .get_unchecked(ent.index()))
            }
        } else {
            None
        }
    }

    /// Returns the mutable arena wrapper.
    ///
    /// # Panics
    ///
    /// Panics if user has not register the arena with type `T`.
    #[inline]
    pub fn arena_mut<T>(&self) -> ArenaWriteGuard<T>
        where T: Component
    {
        ArenaWriteGuard { storage: self.raw_arena::<T>().write().unwrap() }
    }

    /// Returns the immutable arena wrapper.
    ///
    /// # Panics
    ///
    /// Panics if user has not register the arena with type `T`.
    #[inline]
    pub fn arena<T>(&self) -> ArenaReadGuard<T>
        where T: Component
    {
        ArenaReadGuard { storage: self.raw_arena::<T>().read().unwrap() }
    }

    #[inline]
    fn raw_arena<T>(&self) -> &RwLock<T::Storage>
        where T: Component
    {
        Self::any::<T>(
            self.arenas.get(T::type_index())
                .expect("Tried to perform an operation on component type that not registered.")
                .as_ref()
                .expect("Tried to perform an operation on component type that not registered.")
                .as_ref()
        )
    }

    #[inline]
    fn any<T>(v: &Any) -> &RwLock<T::Storage>
        where T: Component
    {
        v.downcast_ref::<RwLock<T::Storage>>().unwrap()
    }
}

pub trait Arena<T>
    where T: Component
{
    fn get(&self, ent: Entity) -> Option<&T>;
    unsafe fn get_unchecked(&self, ent: Entity) -> &T;
}

pub struct ArenaReadGuard<'a, T>
    where T: Component
{
    storage: RwLockReadGuard<'a, T::Storage>,
}

impl<'a, T> Arena<T> for ArenaReadGuard<'a, T>
    where T: Component
{
    #[inline]
    fn get(&self, ent: Entity) -> Option<&T> {
        self.storage.get(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked(&self, ent: Entity) -> &T {
        self.storage.get_unchecked(ent.index())
    }
}

pub trait ArenaMut<T>: Arena<T>
    where T: Component
{
    fn get_mut(&mut self, ent: Entity) -> Option<&mut T>;
    unsafe fn get_unchecked_mut(&mut self, ent: Entity) -> &mut T;
}

pub struct ArenaWriteGuard<'a, T>
    where T: Component
{
    storage: RwLockWriteGuard<'a, T::Storage>,
}

impl<'a, T> Arena<T> for ArenaWriteGuard<'a, T>
    where T: Component
{
    #[inline]
    fn get(&self, ent: Entity) -> Option<&T> {
        self.storage.get(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked(&self, ent: Entity) -> &T {
        self.storage.get_unchecked(ent.index())
    }
}

impl<'a, T> ArenaMut<T> for ArenaWriteGuard<'a, T>
    where T: Component
{
    #[inline]
    fn get_mut(&mut self, ent: Entity) -> Option<&mut T> {
        self.storage.get_mut(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, ent: Entity) -> &mut T {
        self.storage.get_unchecked_mut(ent.index())
    }
}

/// Help builder for entities.
pub struct EntityBuilder<'a> {
    entity: Entity,
    world: &'a mut World,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<T>(&mut self, value: T) -> &mut Self
        where T: Component
    {
        self.world.add::<T>(self.entity, value);
        self
    }

    pub fn with_default<T>(&mut self) -> &mut Self
        where T: Component + Default
    {
        self.world.add_with_default::<T>(self.entity);
        self
    }

    pub fn finish(&self) -> Entity {
        self.entity
    }
}

/// All the implementations of various iterators.
impl World {
    /// Returns immutable `World` iterator into `Entity`s.
    #[inline]
    pub fn iter(&self) -> HandleIter {
        self.entities.iter()
    }
}

build_view_with!(view_with[T1]);
build_view_with!(view_with_2[T1, T2]);
build_view_with!(view_with_3[T1, T2, T3]);
build_view_with!(view_with_4[T1, T2, T3, T4]);
build_view_with!(view_with_5[T1, T2, T3, T4, T5]);
build_view_with!(view_with_6[T1, T2, T3, T4, T5, T6]);
build_view_with!(view_with_7[T1, T2, T3, T4, T5, T6, T7]);
build_view_with!(view_with_8[T1, T2, T3, T4, T5, T6, T7, T8]);

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
