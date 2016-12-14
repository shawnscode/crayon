use std::any::Any;
use std::cell::{Ref, RefMut, RefCell};
use bit_set::BitSet;

use super::*;
use super::super::utility::{HandleIndex, HandleSet, HandleIter};

/// The `World` struct are used to manage the whole entity-component system, It keeps
/// tracks of the state of every created `Entity`s. All memthods are supposed to be
/// valid for any context they are available in.
pub struct World {
    entities: HandleSet,
    masks: Vec<BitSet>,
    erasers: Vec<Box<FnMut(&mut Box<Any>, HandleIndex) -> ()>>,
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
    pub fn create(&mut self) -> Entity {
        let ent = self.entities.create();

        if self.masks.len() <= (ent.index() as usize) {
            self.masks.resize(ent.index() as usize + 1, BitSet::new());
        }

        ent
    }

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
                erase(&mut self.storages[x].as_mut().unwrap(), ent.index())
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
        if T::type_index() >= self.storages.len() {
            for _ in self.storages.len()..(T::type_index() + 1) {
                // Keeps downcast type info in closure.
                let eraser = Box::new(|any: &mut Box<Any>, id: HandleIndex| {
                    unsafe {
                        any.downcast_mut::<RefCell<T::Storage>>().unwrap().borrow_mut().remove(id);
                    }
                });

                self.erasers.push(eraser);
                self.storages.push(None);
            }
        }

        // Returns if we are going to register this component duplicatedly.
        if let Some(_) = self.storages[T::type_index()] {
            return;
        }

        self.storages[T::type_index()] = Some(Box::new(RefCell::new(T::Storage::new())));
    }

    /// Add components to entity, returns the old value if exists.
    pub fn assign<T>(&mut self, ent: Entity, value: T) -> Option<T>
        where T: Component
    {
        unsafe {
            if self.is_alive(ent) {
                let result = if self.masks[ent.index() as usize].contains(T::type_index()) {
                    Some(self._s::<T>().borrow_mut().remove(ent.index()))
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
    }

    pub fn assign_with_default<T>(&mut self, ent: Entity) -> Option<T>
        where T: Component + Default
    {
        self.assign(ent, Default::default())
    }

    /// Remove component of entity from the world, returning the component at the `HandleIndex`.
    pub fn remove<T>(&mut self, ent: Entity) -> Option<T>
        where T: Component
    {
        unsafe {
            if self.masks[ent.index() as usize].contains(T::type_index()) {
                self.masks[ent.index() as usize].remove(T::type_index());
                Some(self._s::<T>().borrow_mut().remove(ent.index()))
            } else {
                None
            }
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
    /// # Panics
    /// Panics if any T is currently mutably borrowed.
    pub fn fetch<T>(&self, ent: Entity) -> Option<Ref<T>>
        where T: Component
    {
        if self.has::<T>(ent) {
            unsafe { Some(Ref::map(self._s::<T>().borrow(), |s| s.get(ent.index()))) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the component corresponding to the `Entity`.
    /// # Panics
    /// Panics if any T is currently borrowed.
    pub fn fetch_mut<T>(&self, ent: Entity) -> Option<RefMut<T>>
        where T: Component
    {
        if self.has::<T>(ent) {
            unsafe { Some(RefMut::map(self._s::<T>().borrow_mut(), |s| s.get_mut(ent.index()))) }
        } else {
            None
        }
    }

    #[inline]
    fn _s<T>(&self) -> &RefCell<T::Storage>
        where T: Component
    {
        self.storages[T::type_index()]
            .as_ref()
            .expect("Tried to perform an operation on component type that not registered.")
            .downcast_ref::<RefCell<T::Storage>>()
            .unwrap()
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
        self.world.assign::<T>(self.entity, value);
        self
    }

    pub fn with_default<T>(&mut self) -> &mut Self
        where T: Component + Default
    {
        self.world.assign_with_default::<T>(self.entity);
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

build_iter_with!(iter_with_r1; RTuple1[T1]; Tuple0[]; [T1]);
build_iter_with!(iter_with_r2; RTuple2[T1, T2]; Tuple0[]; [T1, T2]);
build_iter_with!(iter_with_r3; RTuple3[T1, T2, T3]; Tuple0[]; [T1, T2, T3]);
build_iter_with!(iter_with_r4; RTuple4[T1, T2, T3, T4]; Tuple0[]; [T1, T2, T3, T4]);

build_iter_with!(iter_with_w1; Tuple0[]; WTuple1[T1]; [T1]);
build_iter_with!(iter_with_w2; Tuple0[]; WTuple2[T1, T2]; [T1, T2]);
build_iter_with!(iter_with_w3; Tuple0[]; WTuple3[T1, T2, T3]; [T1, T2, T3]);
build_iter_with!(iter_with_w4; Tuple0[]; WTuple4[T1, T2, T3, T4]; [T1, T2, T3, T4]);

build_iter_with!(iter_with_r1w1; RTuple1[T1]; WTuple1[T2]; [T1, T2]);
build_iter_with!(iter_with_r1w2; RTuple1[T1]; WTuple2[T2, T3]; [T1, T2, T3]);
build_iter_with!(iter_with_r1w3; RTuple1[T1]; WTuple3[T2, T3, T4]; [T1, T2, T3, T4]);
build_iter_with!(iter_with_r1w4; RTuple1[T1]; WTuple4[T2, T3, T4, T5]; [T1, T2, T3, T4, T5]);
build_iter_with!(iter_with_r2w1; RTuple2[T1, T2]; WTuple1[T3]; [T1, T2, T3]);
build_iter_with!(iter_with_r2w2; RTuple2[T1, T2]; WTuple2[T3, T4]; [T1, T2, T3, T4]);
build_iter_with!(iter_with_r2w3; RTuple2[T1, T2]; WTuple3[T3, T4, T5]; [T1, T2, T3, T4, T5]);
build_iter_with!(iter_with_r2w4; RTuple2[T1, T2]; WTuple4[T3, T4, T5, T6]; [T1, T2, T3, T4, T5, T6]);
build_iter_with!(iter_with_r3w1; RTuple3[T1, T2, T3]; WTuple1[T4]; [T1, T2, T3, T4]);
build_iter_with!(iter_with_r3w2; RTuple3[T1, T2, T3]; WTuple2[T4, T5]; [T1, T2, T3, T4, T5]);
build_iter_with!(iter_with_r3w3; RTuple3[T1, T2, T3]; WTuple3[T4, T5, T6]; [T1, T2, T3, T4, T5, T6]);
build_iter_with!(iter_with_r3w4; RTuple3[T1, T2, T3]; WTuple4[T4, T5, T6, T7]; [T1, T2, T3, T4, T5, T6, T7]);
build_iter_with!(iter_with_r4w1; RTuple4[T1, T2, T3, T4]; WTuple1[T5]; [T1, T2, T3, T4, T5]);
build_iter_with!(iter_with_r4w2; RTuple4[T1, T2, T3, T4]; WTuple2[T5, T6]; [T1, T2, T3, T4, T5, T6]);
build_iter_with!(iter_with_r4w3; RTuple4[T1, T2, T3, T4]; WTuple3[T5, T6, T7]; [T1, T2, T3, T4, T5, T6, T7]);
build_iter_with!(iter_with_r4w4; RTuple4[T1, T2, T3, T4]; WTuple4[T5, T6, T7, T8]; [T1, T2, T3, T4, T5, T6, T7, T8]);

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
