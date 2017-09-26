macro_rules! build_view_with {
    ($name: ident[$($cps: ident), *]) => (

        mod $name {
            use bit_set::BitSet;
            use super::*;
            use super::super::{Component, Entity};
            use super::super::super::utils::HandleIter;

            pub struct View<'a> {
                world: &'a World,
                mask: BitSet,
            }

            impl<'a> IntoIterator for View<'a> {
                type Item = Entity;
                type IntoIter = ViewIterator<'a>;

                fn into_iter(self) -> ViewIterator<'a> {
                    let iter = self.world.iter();
                    ViewIterator { view: self, iterator: iter }
                }
            }

            pub struct ViewIterator<'a> {
                view: View<'a>,
                iterator: HandleIter<'a>,
            }

            fn next_item<'a>(view: &View<'a>,
                             iterator: &mut HandleIter<'a>) -> Option<Entity>
            {
                loop {
                    match iterator.next() {
                        Some(ent) => {
                            let mut mask =
                                unsafe { view.world.masks.get_unchecked(ent.index() as usize).clone() };
                            mask.intersect_with(&view.mask);
                            if mask == view.mask {
                                return Some(ent);
                            }
                        }
                        None => {
                            return None;
                        }
                    }
                }
            }

            impl<'a> Iterator for ViewIterator<'a> {
                type Item = Entity;

                fn next(&mut self) -> Option<Self::Item> {
                    unsafe {
                        let iter = &mut self.iterator as *mut HandleIter;
                        next_item(&self.view, &mut *iter)
                    }
                }
            }

            impl<'a> View<'a> {
                pub fn as_slice(&mut self) -> ViewSlice {
                    let iter = self.world.iter();
                    ViewSlice {
                        view: self as *mut View as * mut (),
                        iterator: iter,
                    }
                }
            }

            pub struct ViewSlice<'a> {
                view: *mut (),
                iterator: HandleIter<'a>,
            }

            impl<'a> Iterator for ViewSlice<'a> {
                type Item = Entity;

                fn next(&mut self) -> Option<Self::Item> {
                    unsafe {
                        let iter = &mut self.iterator as *mut HandleIter;
                        let view = &mut *(self.view as *mut View);
                        next_item(view, &mut *iter)
                    }
                }
            }

            unsafe impl<'a> Send for ViewSlice<'a> {}
            unsafe impl<'a> Sync for ViewSlice<'a> {}

            impl<'a> ViewSlice<'a> {
                pub fn split_with(&mut self, len: usize) -> (ViewSlice, ViewSlice) {
                    let (lhs, rhs) = self.iterator.split_with(len);
                    (ViewSlice { view: self.view, iterator: lhs },
                     ViewSlice { view: self.view, iterator: rhs })
                }

                pub fn split(&mut self) -> (ViewSlice, ViewSlice) {
                    let (lhs, rhs) = self.iterator.split();
                    (ViewSlice { view: self.view, iterator: lhs },
                     ViewSlice { view: self.view, iterator: rhs } )
                }
            }

            impl World {
                pub fn $name<$($cps), *>(&self) -> (View, ($(ArenaGetter<$cps>), *))
                    where $($cps:Component, )*
                {
                    let mut mask = BitSet::new();
                    $( mask.insert($cps::type_index()); ) *

                    (
                        View {
                            world: self,
                            mask: mask,
                        },
                        ( $(self.arena::<$cps>().unwrap()), * )
                    )
                }
            }
        }
    )
}
