//! The `World` struct contains entities and its component storages.

use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;

use utils::handle_pool::Iter;
use utils::{HandleIndex, HandlePool};

use ecs::bitset::BitSet;
use ecs::component::{Arena, Component};

/// `Entity` type, as seen by the user, its a alias to `Handle` internally.
pub type Entity = ::utils::handle::Handle;

/// The `World` struct are used to manage the whole entity-component system, It keeps
/// tracks of the state of every created `Entity`s. All memthods are supposed to be
/// valid for any context they are available in.
#[derive(Default)]
pub struct World {
    masks: Vec<BitSet>,
    entities: HandlePool,
    registry: HashMap<TypeId, usize>,
    arenas: ArenaVec,
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
            registry: HashMap::new(),
            arenas: ArenaVec::default(),
        }
    }

    /// Registers a new component type.
    pub fn register<T>(&mut self)
    where
        T: Component,
        T::Arena: Default,
    {
        let id = TypeId::of::<T>();

        // Returns if we are going to register this component duplicatedly.
        if self.registry.contains_key(&id) {
            return;
        }

        self.registry.insert(id, self.arenas.len());
        self.arenas.insert::<T>();
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

    /// Returns true if this `Handle` was created by `HandlePool`, and has not
    /// been freed yet.
    #[inline]
    pub fn is_alive(&self, ent: Entity) -> bool {
        self.entities.is_alive(ent)
    }

    /// Returns the number of current alive entities in this `World`.
    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Checks if the world is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Recycles the `Entity` handle, free corresponding components. and mark
    /// its version as dead.
    pub fn free(&mut self, ent: Entity) -> bool {
        if self.is_alive(ent) {
            unsafe {
                for x in self.masks[ent.index() as usize].iter() {
                    self.arenas.erase(x, ent);
                }
            }

            self.masks[ent.index() as usize].clear();
            self.entities.free(ent)
        } else {
            false
        }
    }

    /// Add components to entity, returns the old value if exists.
    pub fn add<T>(&mut self, ent: Entity, value: T) -> Option<T>
    where
        T: Component,
    {
        let index = self.mask_index::<T>();

        if self.is_alive(ent) {
            self.masks[ent.index() as usize].insert(index);
            unsafe { self.arenas.get_mut::<T>(index).insert(ent.index(), value) }
        } else {
            None
        }
    }

    /// Add a component with default contructed to entity, returns the old value
    /// if exists.
    pub fn add_with_default<T>(&mut self, ent: Entity) -> Option<T>
    where
        T: Component + Default,
    {
        self.add(ent, Default::default())
    }

    /// Remove component of entity from the world, returning the component at the
    /// `HandleIndex`.
    pub fn remove<T>(&mut self, ent: Entity) -> Option<T>
    where
        T: Component,
    {
        let index = self.mask_index::<T>();

        if self.masks[ent.index() as usize].contains(index) {
            self.masks[ent.index() as usize].remove(index);
            unsafe { self.arenas.get_mut::<T>(index).remove(ent.index()) }
        } else {
            None
        }
    }

    /// Returns true if we have componen in this `Entity`, otherwise false.
    #[inline]
    pub fn has<T>(&self, ent: Entity) -> bool
    where
        T: Component,
    {
        let index = self.mask_index::<T>();
        self.entities.is_alive(ent) && self.masks[ent.index() as usize].contains(index)
    }

    /// Returns a reference to the component corresponding to the `Entity`.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple immutable
    /// borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the arena is currently mutably borrowed.
    pub fn get<T>(&self, ent: Entity) -> Option<&T>
    where
        T: Component,
    {
        if self.has::<T>(ent) {
            unsafe { Some(self.arena::<T>().get_unchecked(ent.index())) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the componenent corresponding to the `Entity`.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope.
    pub fn get_mut<T>(&mut self, ent: Entity) -> Option<&mut T>
    where
        T: Component,
    {
        if self.has::<T>(ent) {
            unsafe { Some(self.arena_mut::<T>().get_unchecked_mut(ent.index())) }
        } else {
            None
        }
    }

    /// Gets immutable `World` iterator into all of the `Entity`s.
    #[inline]
    pub fn entities(&self) -> Entities {
        Entities::new(self)
    }
}

/// Help builder for entities.
pub struct EntityBuilder<'a> {
    entity: Entity,
    world: &'a mut World,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<T>(self, value: T) -> Self
    where
        T: Component,
    {
        self.world.add::<T>(self.entity, value);
        self
    }

    pub fn with_default<T>(self) -> Self
    where
        T: Component + Default,
    {
        self.world.add_with_default::<T>(self.entity);
        self
    }

    pub fn finish(self) -> Entity {
        self.entity
    }
}

/// View into entities.
#[derive(Copy, Clone)]
pub struct Entities<'w> {
    world: &'w World,
}

impl<'w> Entities<'w> {
    #[inline]
    fn new(world: &'w World) -> Self {
        Entities { world: world }
    }

    #[inline]
    pub fn index<T: Component>(&self) -> usize {
        self.world.mask_index::<T>()
    }

    #[inline]
    pub fn iter(&self, bits: BitSet) -> EntitiesIter {
        EntitiesIter::new(self.world, bits)
    }
}

impl<'a, 'w: 'a> IntoIterator for &'a Entities<'w> {
    type Item = Entity;
    type IntoIter = EntitiesIter<'w>;

    fn into_iter(self) -> Self::IntoIter {
        EntitiesIter::new(self.world, BitSet::new())
    }
}

impl<'a, 'w: 'a> IntoIterator for &'a mut Entities<'w> {
    type Item = Entity;
    type IntoIter = EntitiesIter<'w>;

    fn into_iter(self) -> Self::IntoIter {
        EntitiesIter::new(self.world, BitSet::new())
    }
}

macro_rules! impl_entities_iter {
    ($METHOD:ident, [$($CMPS: ident), *]) => (
        impl<'w> Entities<'w> {
            pub fn $METHOD<$($CMPS: Component,)*>(&self) -> EntitiesIter {
                let bits = BitSet::from(&[$(self.world.mask_index::<$CMPS>(),)*]);
                EntitiesIter::new(self.world, bits)
            }
        }
    )
}

impl_entities_iter!(with_1, [T1]);
impl_entities_iter!(with_2, [T1, T2]);
impl_entities_iter!(with_3, [T1, T2, T3]);
impl_entities_iter!(with_4, [T1, T2, T3, T4]);
impl_entities_iter!(with_5, [T1, T2, T3, T4, T5]);
impl_entities_iter!(with_6, [T1, T2, T3, T4, T5, T6]);
impl_entities_iter!(with_7, [T1, T2, T3, T4, T5, T6, T7]);
impl_entities_iter!(with_8, [T1, T2, T3, T4, T5, T6, T7, T8]);
impl_entities_iter!(with_9, [T1, T2, T3, T4, T5, T6, T7, T8, T9]);

pub struct EntitiesIter<'w> {
    masks: &'w Vec<BitSet>,
    iter: Iter<'w>,
    bits: BitSet,
}

impl<'w> EntitiesIter<'w> {
    fn new(world: &'w World, bits: BitSet) -> Self {
        EntitiesIter {
            masks: &world.masks,
            iter: world.entities.iter(),
            bits: bits,
        }
    }

    /// Divides iterator into two with specified stripe in the first `Iter`.
    pub fn split_at(&self, len: usize) -> (EntitiesIter<'w>, EntitiesIter<'w>) {
        let (left, right) = self.iter.split_at(len);
        (
            EntitiesIter {
                masks: self.masks,
                iter: left,
                bits: self.bits,
            },
            EntitiesIter {
                masks: self.masks,
                iter: right,
                bits: self.bits,
            },
        )
    }

    /// Divides iterator into two at mid.
    ///
    /// The first will contain all indices from [start, mid) (excluding the index mid itself)
    /// and the second will contain all indices from [mid, end) (excluding the index end itself).
    #[inline]
    pub fn split(&self) -> (EntitiesIter<'w>, EntitiesIter<'w>) {
        self.split_at(self.len() / 2)
    }

    /// Returns the size of indices this iterator could reachs.
    #[inline]
    pub fn len(&self) -> usize {
        self.iter.len()
    }

    /// Checks if the iterator is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<'w> Iterator for EntitiesIter<'w> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            loop {
                match self.iter.next() {
                    Some(ent) => {
                        let mask = self.masks.get_unchecked(ent.index() as usize);
                        if mask.intersect_with(&self.bits) == self.bits {
                            return Some(ent);
                        }
                    }
                    None => {
                        return None;
                    }
                }
            }
        }
    }
}

macro_rules! impl_view_with {
    ($name: ident, [$($readables: ident), *], []) => (
        #[allow(unused_imports)]
        mod $name {
            use $crate::ecs::view::{Fetch};
            use $crate::ecs::world::{World, Entities};
            use $crate::ecs::component::Component;
            use $crate::ecs::bitset::BitSet;

            impl World {
                /// Gets multiple storages and the `Entities` at the same time safely.
                #[allow(unused_mut)]
                pub fn $name<$($readables, )*>(&self) -> (Entities, $(Fetch<$readables>, )*)
                where
                    $($readables:Component, )*
                {
                    unsafe {
                        (
                            Entities::new(self),
                            $( Fetch::<$readables>::new(self), )*
                        )
                    }
                }
            }
        }
    );

    ($name: ident, [$($readables: ident), *], [$($writables: ident), *]) => (
        #[allow(unused_imports)]
        mod $name {
            use $crate::ecs::view::{Fetch, FetchMut};
            use $crate::ecs::world::{World, Entities};
            use $crate::ecs::component::Component;
            use $crate::ecs::bitset::BitSet;

            impl World {
                /// Gets multiple storages mutably and the `Entities` at the same time safely.
                ///
                /// # Pancis
                ///
                /// Panics if the storage is currently mutably borrowed.
                #[allow(unused_mut)]
                pub fn $name<$($readables, )* $($writables, )*>(&mut self) -> (Entities, $(Fetch<$readables>, )* $(FetchMut<$writables>, )*)
                where
                    $($readables:Component, )*
                    $($writables:Component, )*
                {
                    let rbits = BitSet::from(&[$(self.mask_index::<$readables>(),)*]);
                    let mut wbits = BitSet::new();

                    $(
                        let index = self.mask_index::<$writables>();
                        assert!(!wbits.contains(index),
                            "You are trying to borrow arena that is currently mutably borrowed.");
                        wbits.insert(index);
                    ) *

                    assert!(rbits.intersect_with(wbits).is_empty(),
                            "You are trying to borrow arena that is currently mutably borrowed.");

                    unsafe {
                        (
                            Entities::new(self),
                            $( Fetch::<$readables>::new(self), )*
                            $( FetchMut::<$writables>::new(self), )*
                        )
                    }
                }
            }
        }
    );
}

impl_view_with!(view_r1, [R1], []);
impl_view_with!(view_r2, [R1, R2], []);
impl_view_with!(view_r3, [R1, R2, R3], []);
impl_view_with!(view_r4, [R1, R2, R3, R4], []);
impl_view_with!(view_r5, [R1, R2, R3, R4, R5], []);
impl_view_with!(view_r6, [R1, R2, R3, R4, R5, R6], []);
impl_view_with!(view_r7, [R1, R2, R3, R4, R5, R6, R7], []);
impl_view_with!(view_r8, [R1, R2, R3, R4, R5, R6, R7, R8], []);
impl_view_with!(view_r9, [R1, R2, R3, R4, R5, R6, R7, R8, R9], []);

impl_view_with!(view_w1, [], [W1]);
impl_view_with!(view_w2, [], [W1, W2]);
impl_view_with!(view_w3, [], [W1, W2, W3]);
impl_view_with!(view_w4, [], [W1, W2, W3, W4]);
impl_view_with!(view_w5, [], [W1, W2, W3, W4, W5]);
impl_view_with!(view_w6, [], [W1, W2, W3, W4, W5, W6]);
impl_view_with!(view_w7, [], [W1, W2, W3, W4, W5, W6, W7]);
impl_view_with!(view_w8, [], [W1, W2, W3, W4, W5, W6, W7, W8]);
impl_view_with!(view_w9, [], [W1, W2, W3, W4, W5, W6, W7, W8, W9]);

impl_view_with!(view_r1w1, [R1], [W1]);
impl_view_with!(view_r2w1, [R1, R2], [W1]);
impl_view_with!(view_r3w1, [R1, R2, R3], [W1]);
impl_view_with!(view_r4w1, [R1, R2, R3, R4], [W1]);

impl_view_with!(view_r1w2, [R1], [W1, W2]);
impl_view_with!(view_r2w2, [R1, R2], [W1, W2]);
impl_view_with!(view_r3w2, [R1, R2, R3], [W1, W2]);
impl_view_with!(view_r4w2, [R1, R2, R3, R4], [W1, W2]);

impl_view_with!(view_r1w3, [R1], [W1, W2, W3]);
impl_view_with!(view_r2w3, [R1, R2], [W1, W2, W3]);
impl_view_with!(view_r3w3, [R1, R2, R3], [W1, W2, W3]);
impl_view_with!(view_r4w3, [R1, R2, R3, R4], [W1, W2, W3]);

impl_view_with!(view_r1w4, [R1], [W1, W2, W3, W4]);
impl_view_with!(view_r2w4, [R1, R2], [W1, W2, W3, W4]);
impl_view_with!(view_r3w4, [R1, R2, R3], [W1, W2, W3, W4]);
impl_view_with!(view_r4w4, [R1, R2, R3, R4], [W1, W2, W3, W4]);

struct ArenaVec {
    arenas: Vec<Box<UnsafeCell<Any + Send + Sync>>>,
    erases: Vec<Box<UnsafeCell<Fn(&mut Any, HandleIndex) -> () + Send + Sync>>>,
}

impl Default for ArenaVec {
    fn default() -> Self {
        ArenaVec {
            arenas: Vec::new(),
            erases: Vec::new(),
        }
    }
}

impl ArenaVec {
    fn insert<T>(&mut self)
    where
        T: Component,
        T::Arena: Default,
    {
        self.arenas
            .push(Box::new(UnsafeCell::new(T::Arena::default())));

        self.erases.push(Box::new(UnsafeCell::new(
            |any: &mut Any, id: HandleIndex| {
                any.downcast_mut::<T::Arena>().unwrap().remove(id);
            },
        )));
    }

    fn len(&self) -> usize {
        self.arenas.len()
    }

    unsafe fn get<T: Component>(&self, index: usize) -> &T::Arena {
        (&*self.arenas.get_unchecked(index).as_ref().get() as &Any)
            .downcast_ref::<T::Arena>()
            .unwrap()
    }

    unsafe fn get_mut<T: Component>(&self, index: usize) -> &mut T::Arena {
        (&mut *self.arenas.get_unchecked(index).as_ref().get() as &mut Any)
            .downcast_mut::<T::Arena>()
            .unwrap()
    }

    unsafe fn erase(&mut self, index: usize, ent: Entity) {
        (&*self.erases.get_unchecked(index).as_ref().get())(
            &mut *self.arenas.get_unchecked(index).as_ref().get() as &mut Any,
            ent.index(),
        )
    }
}

impl World {
    #[inline]
    pub(crate) fn mask_index<T>(&self) -> usize
    where
        T: Component,
    {
        *self.registry
            .get(&TypeId::of::<T>())
            .expect("Component has NOT been registered.")
    }

    #[inline]
    pub(crate) unsafe fn arena<T>(&self) -> &T::Arena
    where
        T: Component,
    {
        let index = self.mask_index::<T>();
        self.arenas.get::<T>(index)
    }

    #[inline]
    pub(crate) unsafe fn arena_mut<T>(&self) -> &mut T::Arena
    where
        T: Component,
    {
        let index = self.mask_index::<T>();
        self.arenas.get_mut::<T>(index)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut world = World::new();
        assert_eq!(world.len(), 0);

        let e = world.create();
        assert!(world.is_alive(e));
        assert_eq!(world.len(), 1);

        world.free(e);
        assert!(!world.is_alive(e));
        assert_eq!(world.len(), 0);
    }
}
