use std::any::Any;
use std::cell::{Ref, RefMut, RefCell};
use bit_set::BitSet;

use super::*;
use super::super::utility::handle::*;

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
                    any.downcast_mut::<RefCell<T::Storage>>().unwrap().borrow_mut().remove(id);
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

    /// Add components to entity, returns the reference to component.
    /// If the world did not have component with present `HandleIndex`, `None` is returned.
    /// Otherwise the value is updated, and the old value is returned.
    pub fn assign<T>(&mut self, ent: Entity, value: T) -> Option<T>
        where T: Component
    {
        if self.is_alive(ent) {
            self.masks[ent.index() as usize].insert(T::type_index());
            self._s::<T>().borrow_mut().insert(ent.index(), value)
        } else {
            None
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
        if self.is_alive(ent) {
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
    /// # Panics
    /// Panics if any T is currently mutably borrowed.
    pub fn fetch<T>(&self, ent: Entity) -> Option<Ref<T>>
        where T: Component
    {
        if self.has::<T>(ent) {
            Some(Ref::map(self._s::<T>().borrow(), |s| s.get(ent.index()).unwrap()))
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
            Some(RefMut::map(self._s::<T>().borrow_mut(),
                             |s| s.get_mut(ent.index()).unwrap()))
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

macro_rules! build_read_iter_with {
    ($name: ident, $tuple:ident<$($cps: ident), *>) => (

        mod $name {
            use bit_set::BitSet;
            use super::*;
            use super::super::{Component, Entity};
            use super::super::iterator::{IterTupleHelper, $tuple};
            use super::super::super::utility::handle::HandleIter;

            pub struct Iter<'a, $($cps), *>
                where $($cps:Component), *
            {
                world: &'a World,
                mask: BitSet,
                iterator: HandleIter<'a>,
                readers: $tuple<'a, $($cps), *>,
            }

            pub struct IterItem<'a, $($cps), *>
                where $($cps:Component), *
            {
                pub entity: Entity,
                pub readables: ($(&'a $cps), *),
            }

            impl<'a, $($cps), *> Iterator for Iter<'a, $($cps), *>
                where $($cps:Component), *
            {
                type Item = IterItem<'a, $($cps), *>;

                fn next(&mut self) -> Option<Self::Item> {
                    loop {
                        match self.iterator.next() {
                            Some(ent) => {
                                let mut mask =
                                    unsafe { self.world.masks.get_unchecked(ent.index() as usize).clone() };
                                mask.intersect_with(&self.mask);

                                if mask == self.mask {
                                    return Some(IterItem {
                                        entity: ent,
                                        readables: (self.readers.fetch(ent.index())),
                                    });
                                }
                            }
                            None => {
                                return None;
                            }
                        }
                    }
                }
            }

            impl World {
                /// Returns iterator into alive entities with specified components.
                pub fn $name<$($cps), *>(&self) -> Iter<$($cps), *>
                    where $($cps:Component, )*
                {
                    let mut mask = BitSet::new();
                    $(
                        mask.insert($cps::type_index());
                    ) *

                    Iter {
                        world: self,
                        mask: mask,
                        iterator: self.iter(),
                        readers: $tuple($(self._s::<$cps>().borrow()), *),
                    }
                }
            }
        }
    )
}

build_read_iter_with!(iter_with, RTuple1<T1>);
build_read_iter_with!(iter_with_2, RTuple2<T1, T2>);
build_read_iter_with!(iter_with_3, RTuple3<T1, T2, T3>);
build_read_iter_with!(iter_with_4, RTuple4<T1, T2, T3, T4>);

macro_rules! build_write_iter_with {
    ($name: ident, $tuple:ident<$($cps: ident), *>) => (

        mod $name {
            use bit_set::BitSet;
            use super::*;
            use super::super::{Component, Entity};
            use super::super::iterator::{IterMutTupleHelper, $tuple};
            use super::super::super::utility::handle::HandleIter;

            pub struct Iter<'a, $($cps), *>
                where $($cps:Component), *
            {
                world: &'a World,
                mask: BitSet,
                iterator: HandleIter<'a>,
                writers: $tuple<'a, $($cps), *>,
            }

            pub struct IterItem<'a, $($cps), *>
                where $($cps:Component), *
            {
                pub entity: Entity,
                pub writables: ($(&'a mut $cps), *),
            }

            impl<'a, $($cps), *> Iterator for Iter<'a, $($cps), *>
                where $($cps:Component), *
            {
                type Item = IterItem<'a, $($cps), *>;

                fn next(&mut self) -> Option<Self::Item> {
                    loop {
                        match self.iterator.next() {
                            Some(ent) => {
                                let mut mask =
                                    unsafe { self.world.masks.get_unchecked(ent.index() as usize).clone() };
                                mask.intersect_with(&self.mask);

                                if mask == self.mask {
                                    return Some(IterItem {
                                        entity: ent,
                                        writables: (self.writers.fetch(ent.index())),
                                    });
                                }
                            }
                            None => {
                                return None;
                            }
                        }
                    }
                }
            }

            impl World {
                /// Returns iterator into alive entities with specified components.
                pub fn $name<$($cps), *>(&self) -> Iter<$($cps), *>
                    where $($cps:Component, )*
                {
                    let mut mask = BitSet::new();
                    $(
                        mask.insert($cps::type_index());
                    ) *

                    Iter {
                        world: self,
                        mask: mask,
                        iterator: self.iter(),
                        writers: $tuple($(self._s::<$cps>().borrow_mut()), *),
                    }
                }
            }
        }
    )
}

build_write_iter_with!(iter_mut_with, WTuple1<T1>);
build_write_iter_with!(iter_mut_with_2, WTuple2<T1, T2>);
build_write_iter_with!(iter_mut_with_3, WTuple3<T1, T2, T3>);
build_write_iter_with!(iter_mut_with_4, WTuple4<T1, T2, T3, T4>);

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
