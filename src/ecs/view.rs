//! Utilities to iterate over the `World` safely.

use ecs::bitset::BitSet;
use ecs::world::{Entities, EntitiesIter, Entity, World};
use ecs::component::{Arena, Component};

/// A arena with immutable read access into underlying components.
pub trait ArenaGet<T: Component> {
    /// Gets a reference to component `T`.
    fn get(&self, ent: Entity) -> Option<&T>;
    /// Gets a reference to component `T` without doing bounds checking.
    unsafe fn get_unchecked(&self, ent: Entity) -> &T;
}

/// A arena with mutable access into underlying components.
pub trait ArenaGetMut<T: Component>: ArenaGet<T> {
    /// Gets a mutable reference to component `T`.
    fn get_mut(&mut self, ent: Entity) -> Option<&mut T>;
    /// Gets a mutable reference to component `T` without doing bounds checking.
    unsafe fn get_unchecked_mut(&mut self, ent: Entity) -> &mut T;
}

pub struct Fetch<'w, T: Component> {
    arena: &'w T::Arena,
}

impl<'w, T: Component> Fetch<'w, T> {
    pub(crate) unsafe fn new(world: &'w World) -> Self {
        Fetch {
            arena: world.arena::<T>(),
        }
    }
}

impl<'w, T: Component> ArenaGet<T> for Fetch<'w, T> {
    #[inline]
    fn get(&self, ent: Entity) -> Option<&T> {
        self.arena.get(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked(&self, ent: Entity) -> &T {
        self.arena.get_unchecked(ent.index())
    }
}

pub struct FetchMut<'w, T: Component> {
    arena: &'w mut T::Arena,
}

impl<'w, T: Component> FetchMut<'w, T> {
    pub(crate) unsafe fn new(world: &'w World) -> Self {
        FetchMut {
            arena: world.arena_mut::<T>(),
        }
    }
}

impl<'w, T: Component> ArenaGet<T> for FetchMut<'w, T> {
    #[inline]
    fn get(&self, ent: Entity) -> Option<&T> {
        self.arena.get(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked(&self, ent: Entity) -> &T {
        self.arena.get_unchecked(ent.index())
    }
}

impl<'w, T: Component> ArenaGetMut<T> for FetchMut<'w, T> {
    #[inline]
    fn get_mut(&mut self, ent: Entity) -> Option<&mut T> {
        self.arena.get_mut(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, ent: Entity) -> &mut T {
        self.arena.get_unchecked_mut(ent.index())
    }
}

/// `Join` trait is used to provide a convenient way to access entities which
/// have specific components at the same time.
pub trait Join: Sized {
    type Item;
    type ItemWithId;

    /// Gets a iterator over entities and its specified components.
    fn join<'e, 'w: 'e>(self, entities: &'e Entities<'w>) -> JoinIter<'e, Self> {
        unsafe {
            JoinIter {
                iter: entities.iter(Self::mask(entities)),
                values: self,
            }
        }
    }

    /// Gets a iterator over specified components.
    fn components<'e, 'w: 'e>(self, entities: &'e Entities<'w>) -> ComponentIter<'e, Self> {
        unsafe {
            ComponentIter {
                iter: entities.iter(Self::mask(entities)),
                values: self,
            }
        }
    }

    #[doc(hidden)]
    unsafe fn mask<'w>(entities: &Entities<'w>) -> BitSet;
    #[doc(hidden)]
    unsafe fn fetch_unchecked(values: &Self, id: Entity) -> Self::Item;
    #[doc(hidden)]
    unsafe fn fetch_with_id_unchecked(values: &Self, id: Entity) -> Self::ItemWithId;
}

impl<'w, C: Component> Join for Fetch<'w, C> {
    type Item = &'w C;
    type ItemWithId = (Entity, &'w C);

    unsafe fn mask<'e>(entities: &Entities<'e>) -> BitSet {
        BitSet::from(&[entities.index::<C>()])
    }

    unsafe fn fetch_unchecked(values: &Self, id: Entity) -> Self::Item {
        (&*(values as *const Self)).get_unchecked(id)
    }

    unsafe fn fetch_with_id_unchecked(values: &Self, id: Entity) -> Self::ItemWithId {
        (id, (&*(values as *const Self)).get_unchecked(id))
    }
}

impl<'f, 'w: 'f, C: Component> Join for &'f Fetch<'w, C> {
    type Item = &'f C;
    type ItemWithId = (Entity, &'f C);

    unsafe fn mask<'e>(entities: &Entities<'e>) -> BitSet {
        BitSet::from(&[entities.index::<C>()])
    }

    unsafe fn fetch_unchecked(values: &Self, id: Entity) -> Self::Item {
        values.get_unchecked(id)
    }

    unsafe fn fetch_with_id_unchecked(values: &Self, id: Entity) -> Self::ItemWithId {
        (id, values.get_unchecked(id))
    }
}

impl<'w, C: Component> Join for FetchMut<'w, C> {
    type Item = &'w mut C;
    type ItemWithId = (Entity, &'w mut C);

    unsafe fn mask<'e>(entities: &Entities<'e>) -> BitSet {
        BitSet::from(&[entities.index::<C>()])
    }

    unsafe fn fetch_unchecked(values: &Self, id: Entity) -> Self::Item {
        (&mut *(values as *const Self as *mut Self)).get_unchecked_mut(id)
    }

    unsafe fn fetch_with_id_unchecked(values: &Self, id: Entity) -> Self::ItemWithId {
        (
            id,
            (&mut *(values as *const Self as *mut Self)).get_unchecked_mut(id),
        )
    }
}

impl<'f, 'w: 'f, C: Component> Join for &'f FetchMut<'w, C> {
    type Item = &'f C;
    type ItemWithId = (Entity, &'f C);

    unsafe fn mask<'e>(entities: &Entities<'e>) -> BitSet {
        BitSet::from(&[entities.index::<C>()])
    }

    unsafe fn fetch_unchecked(values: &Self, id: Entity) -> Self::Item {
        values.get_unchecked(id)
    }

    unsafe fn fetch_with_id_unchecked(values: &Self, id: Entity) -> Self::ItemWithId {
        (id, values.get_unchecked(id))
    }
}

impl<'f, 'w: 'f, C: Component> Join for &'f mut FetchMut<'w, C> {
    type Item = &'f mut C;
    type ItemWithId = (Entity, &'f mut C);

    unsafe fn mask<'e>(entities: &Entities<'e>) -> BitSet {
        BitSet::from(&[entities.index::<C>()])
    }

    unsafe fn fetch_unchecked(values: &Self, id: Entity) -> Self::Item {
        (&mut *(values as *const Self as *mut Self)).get_unchecked_mut(id)
    }

    unsafe fn fetch_with_id_unchecked(values: &Self, id: Entity) -> Self::ItemWithId {
        (
            id,
            (&mut *(values as *const Self as *mut Self)).get_unchecked_mut(id),
        )
    }
}

macro_rules! impl_join {
    ([$($tps: ident), *]) => (
        impl<$($tps: Join, )*> Join for ( $($tps,)* ) {
            type Item = ( $($tps::Item, ) * );
            type ItemWithId = (Entity, $($tps::Item, ) *);

            unsafe fn mask<'e>(entities: &Entities<'e>) -> BitSet {
                let mut bits = BitSet::new();
                $( bits = bits.union_with( $tps::mask(entities) ); ) *
                bits
            }

            #[allow(non_snake_case)]
            unsafe fn fetch_unchecked(values: &Self, id: Entity) -> Self::Item {
                let &($(ref $tps, )*) = values;
                ( $($tps::fetch_unchecked(&$tps, id), )* )
            }

            #[allow(non_snake_case)]
            unsafe fn fetch_with_id_unchecked(values: &Self, id: Entity) -> Self::ItemWithId {
                let &($(ref $tps, )*) = values;
                ( id, $($tps::fetch_unchecked(&$tps, id), )* )
            }
        }
    );
}

impl_join!([T1]);
impl_join!([T1, T2]);
impl_join!([T1, T2, T3]);
impl_join!([T1, T2, T3, T4]);
impl_join!([T1, T2, T3, T4, T5]);
impl_join!([T1, T2, T3, T4, T5, T6]);
impl_join!([T1, T2, T3, T4, T5, T6, T7]);
impl_join!([T1, T2, T3, T4, T5, T6, T7, T8]);
impl_join!([T1, T2, T3, T4, T5, T6, T7, T8, T9]);

/// The `JoinIter` iterates over a group of entities which have associated
/// `Component`s, and returns the `Entity` and its `Component`s in every
/// iteration.
pub struct JoinIter<'w, J: Join> {
    iter: EntitiesIter<'w>,
    values: J,
}

impl<'w, J: Join> Iterator for JoinIter<'w, J> {
    type Item = J::ItemWithId;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|id| unsafe { J::fetch_with_id_unchecked(&self.values, id) })
    }
}

/// The `ComponentIter` iterates over a group of entities which have associated
/// `Component`s, and returns `Component`s in every iteration.
pub struct ComponentIter<'w, J: Join> {
    iter: EntitiesIter<'w>,
    values: J,
}

impl<'w, J: Join> Iterator for ComponentIter<'w, J> {
    type Item = J::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|id| unsafe { J::fetch_unchecked(&self.values, id) })
    }
}

// /// The parallel version of `JoinIter`.
// pub struct JoinParIter<J: Join> {
//     values: J,
// }

// impl<J: Join + Send> ParallelIterator for JoinParIter<J> {
//     type Item = J::Item;
//     fn drive_unindexed<C>(self, consumer: C) -> C::Result
//     where
//         C: UnindexedConsumer<Self::Item>,
//     {
//     }
// }
