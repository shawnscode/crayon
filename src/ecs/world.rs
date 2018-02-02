//! The `World` struct contains entities and its the component arenas.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use utils::{HandleIndex, HandleIter, HandlePool};

use super::*;
use super::bitset::BitSet;
use super::cell::{Ref, RefCell, RefMut};

/// The `World` struct are used to manage the whole entity-component system, It keeps
/// tracks of the state of every created `Entity`s. All memthods are supposed to be
/// valid for any context they are available in.
#[derive(Default)]
pub struct World {
    masks: Vec<BitSet>,
    entities: HandlePool,

    registry: HashMap<TypeId, usize>,
    arenas: Vec<Entry>,
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
            arenas: Vec::new(),
        }
    }

    /// Registers a new component type.
    pub fn register<T>(&mut self)
    where
        T: Component,
    {
        let id = TypeId::of::<T>();

        // Returns if we are going to register this component duplicatedly.
        if self.registry.contains_key(&id) {
            return;
        }

        self.registry.insert(id, self.arenas.len());
        self.arenas.push(Entry::new::<T>());
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
            for x in self.masks[ent.index() as usize].iter() {
                let v = &mut self.arenas[x];
                let arena = &v.arena;
                let eraser = &mut v.eraser;
                eraser(arena.as_ref(), ent.index());
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
        let index = self.index::<T>();

        if self.is_alive(ent) {
            self.masks[ent.index() as usize].insert(index);
            self.cell::<T>().borrow_mut().insert(ent.index(), value)
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
        let index = self.index::<T>();

        if self.masks[ent.index() as usize].contains(index) {
            self.masks[ent.index() as usize].remove(index);
            self.cell::<T>().borrow_mut().remove(ent.index())
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
        let index = self.index::<T>();
        self.entities.is_alive(ent) && self.masks[ent.index() as usize].contains(index)
    }

    /// Returns a reference to the component corresponding to the `Entity`.
    pub fn get<T>(&self, ent: Entity) -> Option<T>
    where
        T: Component + Copy,
    {
        if self.has::<T>(ent) {
            unsafe { Some(*self.cell::<T>().borrow().get_unchecked(ent.index())) }
        } else {
            None
        }
    }

    /// Mutably borrows the wrapped arena. The borrow lasts until the returned
    /// `FetchMut` exits scope. The value cannot be borrowed while this borrow
    /// is active.
    ///
    /// # Panics
    ///
    /// - Panics if user has not register the arena with type `T`.
    /// - Panics if the value is currently borrowed.
    #[inline]
    pub fn arena_mut<T>(&self) -> FetchMut<T>
    where
        T: Component,
    {
        FetchMut {
            arena: self.cell::<T>().borrow_mut(),
        }
    }

    /// Immutably borrows the arena. The borrow lasts until the returned `Fetch`
    /// exits scope. Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// - Panics if user has not register the arena with type `T`.
    /// - Panics if the value is currently mutably borrowed.
    #[inline]
    pub fn arena<T>(&self) -> Fetch<T>
    where
        T: Component,
    {
        Fetch {
            arena: self.cell::<T>().borrow(),
        }
    }

    /// Gets immutable `World` iterator into all of the `Entity`s.
    #[inline]
    pub fn iter(&self) -> HandleIter {
        self.entities.iter()
    }

    pub(crate) fn index<T>(&self) -> usize
    where
        T: Component,
    {
        *self.registry
            .get(&TypeId::of::<T>())
            .expect("Component has NOT been registered.")
    }

    pub(crate) fn view(&self, mask: BitSet) -> View {
        View {
            world: self,
            mask: mask,
        }
    }

    fn cell<T>(&self) -> &RefCell<T::Arena>
    where
        T: Component,
    {
        let index = self.index::<T>();
        Self::any::<T>(self.arenas[index].arena.as_ref())
    }

    #[inline]
    fn any<T>(v: &Any) -> &RefCell<T::Arena>
    where
        T: Component,
    {
        v.downcast_ref::<RefCell<T::Arena>>().unwrap()
    }
}

struct Entry {
    arena: Box<Any + Send + Sync>,
    eraser: Box<FnMut(&Any, HandleIndex) -> () + Send + Sync>,
}

impl Entry {
    fn new<T>() -> Self
    where
        T: Component,
    {
        let eraser = Box::new(|any: &Any, id: HandleIndex| {
            any.downcast_ref::<RefCell<T::Arena>>()
                .unwrap()
                .borrow_mut()
                .remove(id);
        });
        Entry {
            arena: Box::new(RefCell::new(T::Arena::new())),
            eraser: eraser,
        }
    }
}

pub trait Arena<T>
where
    T: Component,
{
    fn get(&self, ent: Entity) -> Option<&T>;
    unsafe fn get_unchecked(&self, ent: Entity) -> &T;
}

pub struct Fetch<'a, T>
where
    T: Component,
{
    arena: Ref<'a, T::Arena>,
}

impl<'a, T> Arena<T> for Fetch<'a, T>
where
    T: Component,
{
    #[inline]
    fn get(&self, ent: Entity) -> Option<&T> {
        self.arena.get(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked(&self, ent: Entity) -> &T {
        self.arena.get_unchecked(ent.index())
    }
}

pub trait ArenaMut<T>: Arena<T>
where
    T: Component,
{
    fn get_mut(&mut self, ent: Entity) -> Option<&mut T>;
    unsafe fn get_unchecked_mut(&mut self, ent: Entity) -> &mut T;
}

pub struct FetchMut<'a, T>
where
    T: Component,
{
    arena: RefMut<'a, T::Arena>,
}

impl<'a, T> Arena<T> for FetchMut<'a, T>
where
    T: Component,
{
    #[inline]
    fn get(&self, ent: Entity) -> Option<&T> {
        self.arena.get(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked(&self, ent: Entity) -> &T {
        self.arena.get_unchecked(ent.index())
    }
}

impl<'a, T> ArenaMut<T> for FetchMut<'a, T>
where
    T: Component,
{
    #[inline]
    fn get_mut(&mut self, ent: Entity) -> Option<&mut T> {
        self.arena.get_mut(ent.index())
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, ent: Entity) -> &mut T {
        self.arena.get_unchecked_mut(ent.index())
    }
}

/// Help builder for entities.
pub struct EntityBuilder<'a> {
    entity: Entity,
    world: &'a mut World,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<T>(&mut self, value: T) -> &mut Self
    where
        T: Component,
    {
        self.world.add::<T>(self.entity, value);
        self
    }

    pub fn with_default<T>(&mut self) -> &mut Self
    where
        T: Component + Default,
    {
        self.world.add_with_default::<T>(self.entity);
        self
    }

    pub fn finish(&self) -> Entity {
        self.entity
    }
}

/// A `View` of the underlying `Entities` in `World` with specified components.
pub struct View<'a> {
    world: &'a World,
    mask: BitSet,
}

impl<'a> IntoIterator for View<'a> {
    type Item = Entity;
    type IntoIter = ViewIter<'a>;

    fn into_iter(self) -> ViewIter<'a> {
        let iter = self.world.iter();
        ViewIter {
            view: self,
            iterator: iter,
        }
    }
}

/// A iterator of all the entities in the `View`.
pub struct ViewIter<'a> {
    view: View<'a>,
    iterator: HandleIter<'a>,
}

fn next_item<'a>(view: &View<'a>, iterator: &mut HandleIter<'a>) -> Option<Entity> {
    loop {
        match iterator.next() {
            Some(ent) => {
                let mask = unsafe { *view.world.masks.get_unchecked(ent.index() as usize) };

                if mask.intersect_with(&view.mask) == view.mask {
                    return Some(ent);
                }
            }
            None => {
                return None;
            }
        }
    }
}

impl<'a> Iterator for ViewIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let iter = &mut self.iterator as *mut HandleIter;
            next_item(&self.view, &mut *iter)
        }
    }
}

impl<'a> View<'a> {
    /// View the underlying `Entities` in some range.
    ///
    /// This has the same lifetime as the original `View`, and so the iterator can continue
    /// to be used while this exists.
    pub fn as_slice(&mut self) -> ViewSlice {
        let iter = self.world.iter();
        ViewSlice {
            view: self as *mut View as *mut (),
            iterator: iter,
        }
    }
}

/// A dynamically-ranged iterator into a `View`
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
    /// Divides one slice into two at an index.
    ///
    /// The first will contain all indices from [0, mid) (excluding the index mid itself) and
    /// the second will contain all indices from [mid, len) (excluding the index len itself).
    pub fn split_at(&mut self, len: usize) -> (ViewSlice, ViewSlice) {
        let (lhs, rhs) = self.iterator.split_at(len);
        (
            ViewSlice {
                view: self.view,
                iterator: lhs,
            },
            ViewSlice {
                view: self.view,
                iterator: rhs,
            },
        )
    }

    /// Divides one slice into two at mid.
    pub fn split(&mut self) -> (ViewSlice, ViewSlice) {
        let (lhs, rhs) = self.iterator.split();
        (
            ViewSlice {
                view: self.view,
                iterator: lhs,
            },
            ViewSlice {
                view: self.view,
                iterator: rhs,
            },
        )
    }
}

macro_rules! build_view_with {
    ($name: ident[$($cps: ident), *]) => (
        mod $name {
            use $crate::ecs::*;

            impl World {
                pub fn $name<$($cps), *>(&self) -> (View, ($(FetchMut<$cps>), *))
                    where $($cps:Component, )*
                {
                    let mut mask = $crate::ecs::bitset::BitSet::new();
                    $( mask.insert(self.index::<$cps>()); ) *

                    (
                        View {
                            world: self,
                            mask: mask,
                        },
                        ( $(self.arena_mut::<$cps>()), * )
                    )
                }
            }
        }
    )
}

build_view_with!(view_with[T1]);
build_view_with!(view_with_2[T1, T2]);
build_view_with!(view_with_3[T1, T2, T3]);
build_view_with!(view_with_4[T1, T2, T3, T4]);
build_view_with!(view_with_5[T1, T2, T3, T4, T5]);
build_view_with!(view_with_6[T1, T2, T3, T4, T5, T6]);
build_view_with!(view_with_7[T1, T2, T3, T4, T5, T6, T7]);
build_view_with!(view_with_8[T1, T2, T3, T4, T5, T6, T7, T8]);
build_view_with!(view_with_9[T1, T2, T3, T4, T5, T6, T7, T8, T9]);

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
