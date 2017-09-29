//! The `World` struct contains entities and its the component arenas.

use std::any::Any;
use std::borrow::Borrow;
use std::cell::{Ref, RefMut, RefCell};
use bit_set::BitSet;

use super::*;
use super::super::utils::{HandleIndex, HandleSet, HandleIter};

/// The `World` struct are used to manage the whole entity-component system, It keeps
/// tracks of the state of every created `Entity`s. All memthods are supposed to be
/// valid for any context they are available in.
pub struct World {
    entities: HandleSet,
    masks: Vec<BitSet>,
    erasers: Vec<Box<FnMut(&mut Box<Any>, HandleIndex) -> ()>>,
    arenas: Vec<Option<Box<Any>>>,
}

impl World {
    /// Constructs a new empty `World`.
    pub fn new() -> Self {
        World {
            entities: HandleSet::new(),
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
    pub fn free(&mut self, ent: Entity) -> bool {
        if self.is_alive(ent) {
            for x in self.masks[ent.index() as usize].iter() {
                let erase = &mut self.erasers[x];
                erase(&mut self.arenas[x].as_mut().unwrap(), ent.index())
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
                let eraser = Box::new(|any: &mut Box<Any>, id: HandleIndex| {
                                          any.downcast_mut::<RefCell<T::Storage>>()
                                              .unwrap()
                                              .borrow_mut()
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

        self.arenas[T::type_index()] = Some(Box::new(RefCell::new(T::Storage::new())));
    }

    /// Add components to entity, returns the old value if exists.
    pub fn add<T>(&mut self, ent: Entity, value: T) -> Option<T>
        where T: Component
    {
        if self.is_alive(ent) {
            let result = if self.masks[ent.index() as usize].contains(T::type_index()) {
                self._s::<T>().borrow_mut().remove(ent.index())
            } else {
                self.masks[ent.index() as usize].insert(T::type_index());
                None
            };

            self._s::<T>().borrow_mut().insert(ent.index(), value);
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
            self._s::<T>().borrow_mut().remove(ent.index())
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
    ///
    /// # Panics
    /// Panics if any T is currently mutably borrowed.
    pub fn get<T>(&self, ent: Entity) -> Option<Ref<T>>
        where T: Component
    {
        if self.has::<T>(ent) {
            unsafe { Some(Ref::map(self._s::<T>().borrow(), |s| s.get_unchecked(ent.index()))) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the component corresponding to the `Entity`.
    /// # Panics
    /// Panics if any T is currently borrowed.
    pub fn get_mut<T>(&self, ent: Entity) -> Option<RefMut<T>>
        where T: Component
    {
        if self.has::<T>(ent) {
            unsafe {
                Some(RefMut::map(self._s::<T>().borrow_mut(),
                                 |s| s.get_unchecked_mut(ent.index())))
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn arena_mut<T>(&self) -> Option<ArenaMutGetter<T>>
        where T: Component
    {
        if let Some(element) = self.arenas.get(T::type_index()) {
            if let Some(ref s) = *element {
                let v = s.downcast_ref::<RefCell<T::Storage>>().unwrap();
                return Some(ArenaMutGetter { storage: v.borrow_mut() });
            }
        }

        None
    }

    #[inline]
    fn _s<T>(&self) -> &RefCell<T::Storage>
        where T: Component
    {
        self.arenas[T::type_index()]
            .as_ref()
            .expect("Tried to perform an operation on component type that not registered.")
            .downcast_ref::<RefCell<T::Storage>>()
            .unwrap()
    }
}

pub struct ArenaMutGetter<'a, T>
    where T: Component
{
    storage: RefMut<'a, T::Storage>,
}

impl<'a, T> ArenaMutGetter<'a, T>
    where T: Component
{
    #[inline]
    pub fn get<U>(&self, index: U) -> Option<&T>
        where U: Borrow<HandleIndex>
    {
        self.storage.get(*index.borrow())
    }

    #[inline]
    pub unsafe fn get_unchecked<U>(&self, index: U) -> &T
        where U: Borrow<HandleIndex>
    {
        self.storage.get_unchecked(*index.borrow())
    }

    #[inline]
    pub fn get_mut<U>(&mut self, index: U) -> Option<&mut T>
        where U: Borrow<HandleIndex>
    {
        self.storage.get_mut(*index.borrow())
    }

    #[inline]
    pub unsafe fn get_unchecked_mut<U>(&mut self, index: U) -> &mut T
        where U: Borrow<HandleIndex>
    {
        self.storage.get_unchecked_mut(*index.borrow())
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
