use std::cell::{Ref, RefMut};

use super::component::{Component, ComponentStorage};
use super::super::utility::handle::HandleIndex;

pub trait IterTupleHelper {
    type Item;

    fn fetch(&self, index: HandleIndex) -> Self::Item;
}

unsafe fn _cast<'a, T>(value: &T::Storage) -> &'a T::Storage
    where T: Component
{
    ::std::mem::transmute::<&T::Storage, &'a T::Storage>(value)
}

pub struct RTuple1<'a, T1>(pub Ref<'a, T1::Storage>) where T1: Component;

impl<'a, T1> IterTupleHelper for RTuple1<'a, T1>
    where T1: Component
{
    type Item = (&'a T1);

    fn fetch(&self, index: HandleIndex) -> Self::Item {
        unsafe { (_cast::<T1>(&*self.0).get(index)) }
    }
}

pub struct RTuple2<'a, T1, T2>(pub Ref<'a, T1::Storage>, pub Ref<'a, T2::Storage>)
    where T1: Component,
          T2: Component;

impl<'a, T1, T2> IterTupleHelper for RTuple2<'a, T1, T2>
    where T1: Component,
          T2: Component
{
    type Item = (&'a T1, &'a T2);

    fn fetch(&self, index: HandleIndex) -> Self::Item {
        unsafe { (_cast::<T1>(&*self.0).get(index), _cast::<T2>(&*self.1).get(index)) }
    }
}

pub struct RTuple3<'a, T1, T2, T3>(pub Ref<'a, T1::Storage>,
                                   pub Ref<'a, T2::Storage>,
                                   pub Ref<'a, T3::Storage>)
    where T1: Component,
          T2: Component,
          T3: Component;

impl<'a, T1, T2, T3> IterTupleHelper for RTuple3<'a, T1, T2, T3>
    where T1: Component,
          T2: Component,
          T3: Component
{
    type Item = (&'a T1, &'a T2, &'a T3);

    fn fetch(&self, index: HandleIndex) -> Self::Item {
        unsafe {
            (_cast::<T1>(&*self.0).get(index),
             _cast::<T2>(&*self.1).get(index),
             _cast::<T3>(&*self.2).get(index))
        }
    }
}

pub struct RTuple4<'a, T1, T2, T3, T4>(pub Ref<'a, T1::Storage>,
                                       pub Ref<'a, T2::Storage>,
                                       pub Ref<'a, T3::Storage>,
                                       pub Ref<'a, T4::Storage>)
    where T1: Component,
          T2: Component,
          T3: Component,
          T4: Component;

impl<'a, T1, T2, T3, T4> IterTupleHelper for RTuple4<'a, T1, T2, T3, T4>
    where T1: Component,
          T2: Component,
          T3: Component,
          T4: Component
{
    type Item = (&'a T1, &'a T2, &'a T3, &'a T4);

    fn fetch(&self, index: HandleIndex) -> Self::Item {
        unsafe {
            (_cast::<T1>(&*self.0).get(index),
             _cast::<T2>(&*self.1).get(index),
             _cast::<T3>(&*self.2).get(index),
             _cast::<T4>(&*self.3).get(index))
        }
    }
}

///
pub trait IterMutTupleHelper {
    type Item;

    fn fetch(&mut self, index: HandleIndex) -> Self::Item;
}

unsafe fn _cast_mut<'a, T>(value: &mut T::Storage) -> &'a mut T::Storage
    where T: Component
{
    ::std::mem::transmute::<&mut T::Storage, &'a mut T::Storage>(value)
}

pub struct WTuple1<'a, T1>(pub RefMut<'a, T1::Storage>) where T1: Component;

impl<'a, T1> IterMutTupleHelper for WTuple1<'a, T1>
    where T1: Component
{
    type Item = (&'a mut T1);

    fn fetch(&mut self, index: HandleIndex) -> Self::Item {
        unsafe { (_cast_mut::<T1>(&mut *self.0).get_mut(index)) }
    }
}

pub struct WTuple2<'a, T1, T2>(pub RefMut<'a, T1::Storage>, pub RefMut<'a, T2::Storage>)
    where T1: Component,
          T2: Component;

impl<'a, T1, T2> IterMutTupleHelper for WTuple2<'a, T1, T2>
    where T1: Component,
          T2: Component
{
    type Item = (&'a mut T1, &'a mut T2);

    fn fetch(&mut self, index: HandleIndex) -> Self::Item {
        unsafe {
            (_cast_mut::<T1>(&mut *self.0).get_mut(index),
             _cast_mut::<T2>(&mut *self.1).get_mut(index))
        }
    }
}

pub struct WTuple3<'a, T1, T2, T3>(pub RefMut<'a, T1::Storage>,
                                   pub RefMut<'a, T2::Storage>,
                                   pub RefMut<'a, T3::Storage>)
    where T1: Component,
          T2: Component,
          T3: Component;

impl<'a, T1, T2, T3> IterMutTupleHelper for WTuple3<'a, T1, T2, T3>
    where T1: Component,
          T2: Component,
          T3: Component
{
    type Item = (&'a mut T1, &'a mut T2, &'a mut T3);

    fn fetch(&mut self, index: HandleIndex) -> Self::Item {
        unsafe {
            (_cast_mut::<T1>(&mut *self.0).get_mut(index),
             _cast_mut::<T2>(&mut *self.1).get_mut(index),
             _cast_mut::<T3>(&mut *self.2).get_mut(index))
        }
    }
}

pub struct WTuple4<'a, T1, T2, T3, T4>(pub RefMut<'a, T1::Storage>,
                                       pub RefMut<'a, T2::Storage>,
                                       pub RefMut<'a, T3::Storage>,
                                       pub RefMut<'a, T4::Storage>)
    where T1: Component,
          T2: Component,
          T3: Component,
          T4: Component;

impl<'a, T1, T2, T3, T4> IterMutTupleHelper for WTuple4<'a, T1, T2, T3, T4>
    where T1: Component,
          T2: Component,
          T3: Component,
          T4: Component
{
    type Item = (&'a mut T1, &'a mut T2, &'a mut T3, &'a mut T4);

    fn fetch(&mut self, index: HandleIndex) -> Self::Item {
        unsafe {
            (_cast_mut::<T1>(&mut *self.0).get_mut(index),
             _cast_mut::<T2>(&mut *self.1).get_mut(index),
             _cast_mut::<T3>(&mut *self.2).get_mut(index),
             _cast_mut::<T4>(&mut *self.3).get_mut(index))
        }
    }
}

macro_rules! build_read_iter_with {
    ($name: ident, $tuple:ident<$($cps: ident), *>) => (

        mod $name {
            use bit_set::BitSet;
            use super::*;
            use super::super::{Component, Entity};
            use super::super::iterator::{IterTupleHelper, $tuple};
            use super::super::super::utility::HandleIter;

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

macro_rules! build_write_iter_with {
    ($name: ident, $tuple:ident<$($cps: ident), *>) => (

        mod $name {
            use bit_set::BitSet;
            use super::*;
            use super::super::{Component, Entity};
            use super::super::iterator::{IterMutTupleHelper, $tuple};
            use super::super::super::utility::HandleIter;

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

macro_rules! build_iter_with {
    ($name: ident, $rtuple:ident<$($rcps: ident), *>, $wtuple:ident<$($wcps: ident), *>) => (

        mod $name {
            use bit_set::BitSet;
            use super::*;
            use super::super::{Component, Entity};
            use super::super::iterator::{IterMutTupleHelper, IterTupleHelper, $rtuple, $wtuple};
            use super::super::super::utility::HandleIter;

            pub struct Iter<'a, $($rcps), *, $($wcps), *>
                where $($rcps:Component), *,
                      $($wcps:Component), *
                    
            {
                world: &'a World,
                mask: BitSet,
                iterator: HandleIter<'a>,
                readers: $rtuple<'a, $($rcps), *>,
                writers: $wtuple<'a, $($wcps), *>,
            }

            pub struct IterItem<'a, $($rcps), *, $($wcps), *>
                where $($rcps:Component), *,
                      $($wcps:Component), *
            {
                pub entity: Entity,
                pub readables: ($(&'a $rcps), *),
                pub writables: ($(&'a mut $wcps), *),
            }

            impl<'a, $($rcps), *, $($wcps), *> Iterator for Iter<'a, $($rcps), *, $($wcps), *>
                where $($rcps:Component), *,
                      $($wcps:Component), *
            {
                type Item = IterItem<'a, $($rcps), *, $($wcps), *>;

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
                pub fn $name<$($rcps), *, $($wcps), *>(&self) -> Iter<$($rcps), *, $($wcps), *>
                    where $($rcps:Component, )*
                          $($wcps:Component, )*
                {
                    let mut mask = BitSet::new();

                    $( mask.insert($rcps::type_index()); ) *
                    $( mask.insert($wcps::type_index()); ) *

                    Iter {
                        world: self,
                        mask: mask,
                        iterator: self.iter(),
                        readers: $rtuple($(self._s::<$rcps>().borrow()), *),
                        writers: $wtuple($(self._s::<$wcps>().borrow_mut()), *),
                    }
                }
            }
        }
    )
}
