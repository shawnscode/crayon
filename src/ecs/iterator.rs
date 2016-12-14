use std::cell::{Ref, RefMut};
use std::marker::PhantomData;

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

pub struct Tuple0<'a>(pub PhantomData<&'a bool>);

impl<'a> IterTupleHelper for Tuple0<'a> {
    type Item = ();

    fn fetch(&self, _: HandleIndex) -> Self::Item {}
}


pub struct RTuple1<'a, T1>(pub PhantomData<&'a bool>, pub Ref<'a, T1::Storage>) where T1: Component;

impl<'a, T1> IterTupleHelper for RTuple1<'a, T1>
    where T1: Component
{
    type Item = (&'a T1);

    fn fetch(&self, index: HandleIndex) -> Self::Item {
        unsafe { (_cast::<T1>(&*self.1).get(index)) }
    }
}

pub struct RTuple2<'a, T1, T2>(pub PhantomData<&'a bool>,
                               pub Ref<'a, T1::Storage>,
                               pub Ref<'a, T2::Storage>)
    where T1: Component,
          T2: Component;

impl<'a, T1, T2> IterTupleHelper for RTuple2<'a, T1, T2>
    where T1: Component,
          T2: Component
{
    type Item = (&'a T1, &'a T2);

    fn fetch(&self, index: HandleIndex) -> Self::Item {
        unsafe { (_cast::<T1>(&*self.1).get(index), _cast::<T2>(&*self.2).get(index)) }
    }
}

pub struct RTuple3<'a, T1, T2, T3>(pub PhantomData<&'a bool>,
                                   pub Ref<'a, T1::Storage>,
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
            (_cast::<T1>(&*self.1).get(index),
             _cast::<T2>(&*self.2).get(index),
             _cast::<T3>(&*self.3).get(index))
        }
    }
}

pub struct RTuple4<'a, T1, T2, T3, T4>(pub PhantomData<&'a bool>,
                                       pub Ref<'a, T1::Storage>,
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
            (_cast::<T1>(&*self.1).get(index),
             _cast::<T2>(&*self.2).get(index),
             _cast::<T3>(&*self.3).get(index),
             _cast::<T4>(&*self.4).get(index))
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

pub struct WTuple1<'a, T1>(pub PhantomData<&'a bool>, pub RefMut<'a, T1::Storage>)
    where T1: Component;

impl<'a, T1> IterMutTupleHelper for WTuple1<'a, T1>
    where T1: Component
{
    type Item = (&'a mut T1);

    fn fetch(&mut self, index: HandleIndex) -> Self::Item {
        unsafe { (_cast_mut::<T1>(&mut *self.1).get_mut(index)) }
    }
}

pub struct WTuple2<'a, T1, T2>(pub PhantomData<&'a bool>,
                               pub RefMut<'a, T1::Storage>,
                               pub RefMut<'a, T2::Storage>)
    where T1: Component,
          T2: Component;

impl<'a, T1, T2> IterMutTupleHelper for WTuple2<'a, T1, T2>
    where T1: Component,
          T2: Component
{
    type Item = (&'a mut T1, &'a mut T2);

    fn fetch(&mut self, index: HandleIndex) -> Self::Item {
        unsafe {
            (_cast_mut::<T1>(&mut *self.1).get_mut(index),
             _cast_mut::<T2>(&mut *self.2).get_mut(index))
        }
    }
}

pub struct WTuple3<'a, T1, T2, T3>(pub PhantomData<&'a bool>,
                                   pub RefMut<'a, T1::Storage>,
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
            (_cast_mut::<T1>(&mut *self.1).get_mut(index),
             _cast_mut::<T2>(&mut *self.2).get_mut(index),
             _cast_mut::<T3>(&mut *self.3).get_mut(index))
        }
    }
}

pub struct WTuple4<'a, T1, T2, T3, T4>(pub PhantomData<&'a bool>,
                                       pub RefMut<'a, T1::Storage>,
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
            (_cast_mut::<T1>(&mut *self.1).get_mut(index),
             _cast_mut::<T2>(&mut *self.2).get_mut(index),
             _cast_mut::<T3>(&mut *self.3).get_mut(index),
             _cast_mut::<T4>(&mut *self.4).get_mut(index))
        }
    }
}

macro_rules! build_view_with {
    ($name: ident; $rtuple:ident[$($rcps: ident), *]; $wtuple:ident[$($wcps: ident), *]; [$($cps: ident), *]) => (

        mod $name {
            use std::marker::PhantomData;
            use bit_set::BitSet;
            use super::*;
            use super::super::{Component, Entity};
            use super::super::iterator::*;
            use super::super::super::utility::HandleIter;

            pub struct View<'a, $($cps), *>
                where $($cps:Component), *
            {
                world: &'a World,
                mask: BitSet,
                readers: $rtuple<'a, $($rcps), *>,
                writers: $wtuple<'a, $($wcps), *>,
            }

            pub struct ViewItem<'a, $($cps), *>
                where $($cps:Component), *
            {
                pub entity: Entity,
                pub readables: ($(&'a $rcps), *),
                pub writables: ($(&'a mut $wcps), *),
            }

            impl<'a, $($cps), *> IntoIterator for View<'a, $($cps), *>
                where $($cps:Component), *
            {
                type Item = ViewItem<'a, $($cps), *>;
                type IntoIter = IntoIter<'a, $($cps), *>;
                 
                fn into_iter(self) -> Self::IntoIter {
                    let iter = self.world.iter();
                    IntoIter { view: self, iterator: iter }
                }
            }

            impl<'a, $($cps), *> View<'a, $($cps), *>
                where $($cps:Component), *
            {
                pub fn as_slice(&'a mut self) -> ViewSlice<'a, $($cps), *> {
                    let iter = self.world.iter();
                    ViewSlice { view: self, iterator: iter }
                }
            }

            pub struct IntoIter<'a, $($cps), *>
                where $($cps:Component), *
            {
                view: View<'a, $($cps), *>,
                iterator: HandleIter<'a>,
            }

            fn next_item<'a, $($cps), *>(view: &mut View<'a, $($cps), *>,
                                         iterator: &mut HandleIter<'a>) -> Option<ViewItem<'a, $($cps), *>>
                where $($cps:Component), *
            {
                loop {
                    match iterator.next() {
                        Some(ent) => {
                            let mut mask =
                                unsafe { view.world.masks.get_unchecked(ent.index() as usize).clone() };
                            mask.intersect_with(&view.mask);

                            if mask == view.mask {
                                return Some(ViewItem {
                                    entity: ent,
                                    readables: (view.readers.fetch(ent.index())),
                                    writables: (view.writers.fetch(ent.index())),
                                });
                            }
                        }
                        None => {
                            return None;
                        }
                    }
                }                   
            }

            impl<'a, $($cps), *> Iterator for IntoIter<'a, $($cps), *>
                where $($cps:Component), *
            {
                type Item = ViewItem<'a, $($cps), *>;

                fn next(&mut self) -> Option<Self::Item> {
                    unsafe {
                        let iter = &mut self.iterator as *mut HandleIter;
                        next_item(&mut self.view, &mut *iter)
                    }
                }
            }

            pub struct ViewSlice<'a, $($cps), *>
                where $($cps:Component), *
            {
                view: &'a mut View<'a, $($cps), *>,
                iterator: HandleIter<'a>,
            }

            impl<'a, $($cps), *> Iterator for ViewSlice<'a, $($cps), *>
                where $($cps:Component), *
            {
                type Item = ViewItem<'a, $($cps), *>;

                fn next(&mut self) -> Option<Self::Item> {
                    unsafe {
                        let iter = &mut self.iterator as *mut HandleIter;
                        next_item(self.view, &mut *iter)
                    }
                }
            }

            impl<'a, $($cps), *> ViewSlice<'a, $($cps), *>
                where $($cps:Component), *
            {
                pub fn split_with(&'a mut self, len: usize) -> (ViewSlice<'a, $($cps), *>, ViewSlice<'a, $($cps), *>) {
                    unsafe {
                        let (lhs, rhs) = self.iterator.split_with(len);
                        let view = self.view as *mut View<'a, $($cps), *>;
                        (ViewSlice { view: &mut *view, iterator: lhs },
                         ViewSlice { view: &mut *view, iterator: rhs })
                    }
                }

                pub fn split(&'a mut self) -> (ViewSlice<'a, $($cps), *>, ViewSlice<'a, $($cps), *>) {
                    unsafe {
                        let (lhs, rhs) = self.iterator.split();
                        let view = self.view as *mut View<'a, $($cps), *>;
                        (ViewSlice { view: &mut *view, iterator: lhs },
                         ViewSlice { view: &mut *view, iterator: rhs })
                    }
                }
            }

            impl World {
                /// Returns iterator into alive entities with specified components.
                pub fn $name<$($cps), *>(&self) -> View<$($cps), *>
                    where $($cps:Component, )*
                {
                    let mut mask = BitSet::new();

                    $( mask.insert($rcps::type_index()); ) *
                    $( mask.insert($wcps::type_index()); ) *

                    View {
                        world: self,
                        mask: mask,
                        readers: $rtuple(PhantomData, $(self._s::<$rcps>().borrow()), *),
                        writers: $wtuple(PhantomData, $(self._s::<$wcps>().borrow_mut()), *),
                    }
                }
            }
        }
    )
}
