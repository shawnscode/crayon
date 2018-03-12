//! Execution utilities based on `View` and `Arena`s.

use ecs::component::Component;
use ecs::world::{Fetch, FetchMut, View, World};
use ecs::bitset::BitSet;

/// A system that handles `Entities` with specified view.
///
/// # Example
///
/// ```rust,ignore
/// impl ecs::System for Renderer {
///     type ViewWith = (Fetch<Transform>, FetchMut<Mesh>);
///     fn run(&mut self, view: View, arenas: Self::ViewWith) {
///         for ent in view {
///             unsafe {
///                 let transform = arenas.0.get_unchecked(ent);
///                 let mut mesh = arenas.1.get_unchecked_mut(ent);
///             }
///         }
///     }
/// }
pub trait System<'a>: SystemMut<'a> {
    type ViewWith: SystemData<'a> + SystemDataMask;
    type Result: Sized;

    /// Run the system with the required components.
    fn run(&mut self, _: View, _: Self::ViewWith) -> Self::Result {
        unimplemented!()
    }

    fn run_at(&mut self, world: &'a World) -> Self::Result {
        let mask = Self::mask_at(world);
        self.run(world.view(mask), Self::ViewWith::fetch(world))
    }
}

impl<'a, T> SystemMut<'a> for T
where
    T: System<'a>,
{
    type ViewWithMut = <T as System<'a>>::ViewWith;
    type ResultMut = <T as System<'a>>::Result;
}

pub trait SystemMut<'a> {
    /// The component arenas required to execute system.
    type ViewWithMut: SystemDataMut<'a> + SystemDataMask;

    /// The result of execution.
    type ResultMut: Sized;

    /// Mutably Run the system with the required components.
    fn run_mut(&mut self, _: View, _: Self::ViewWithMut) -> Self::ResultMut {
        unimplemented!()
    }

    fn run_mut_at(&mut self, world: &'a mut World) -> Self::ResultMut {
        let mask = Self::mask_at(world);
        self.run_mut(world.view(mask), Self::ViewWithMut::fetch_mut(world))
    }

    fn mask_at(world: &'a World) -> BitSet {
        let r = Self::ViewWithMut::readables(world);
        let w = Self::ViewWithMut::writables(world);
        r.union_with(w)
    }
}

// /// Trait for validation system.
// #[doc(hidden)]
// pub trait SystemValidator {
//     fn readables(&self, world: &World) -> BitSet;
//     fn writables(&self, world: &World) -> BitSet;
// }

// impl<'a, T> SystemValidator for T
// where
//     T: System<'a>,
// {
//     fn readables(&self, world: &World) -> BitSet {
//         T::ViewWith::readables(world)
//     }

//     fn writables(&self, world: &World) -> BitSet {
//         T::ViewWith::writables(world)
//     }
// }

// /// Returns true if the systems could run at the same time safely.
// pub fn validate<'a>(world: &'a World, systems: &[&SystemValidator]) -> bool {
//     let mut r = BitSet::new();
//     let mut w = BitSet::new();

//     for s in systems {
//         r = r.union_with(s.readables(world));

//         if !w.intersect_with(s.writables(world)).is_empty() {
//             return false;
//         }

//         w = w.union_with(s.writables(world));
//     }

//     w.intersect_with(r).is_empty()
// }

pub trait SystemDataMask {
    /// Gets the mask of readable component arenas.
    fn readables(world: &World) -> BitSet;
    /// Gets the mask of writable component arenas.
    fn writables(world: &World) -> BitSet;
}

/// A struct implementing `SystemData` indicates that it bundles some arenas which
/// are required for execution.
pub trait SystemData<'a>: SystemDataMut<'a> {
    /// Creates a new arena bundle by fetching from `World`.
    fn fetch(world: &'a World) -> Self;
}

/// A struct implementing `SystemData` indicates that it bundles some mutable arenas which
/// are required for execution.
pub trait SystemDataMut<'a> {
    /// Creates a new arena bundle by fetching from `World`.
    fn fetch_mut(world: &'a World) -> Self;
}

impl<'a> SystemData<'a> for () {
    fn fetch(_: &'a World) -> Self {
        ()
    }
}

impl<'a> SystemDataMut<'a> for () {
    fn fetch_mut(_: &'a World) -> Self {
        ()
    }
}

impl SystemDataMask for () {
    fn readables(_: &World) -> BitSet {
        BitSet::new()
    }

    fn writables(_: &World) -> BitSet {
        BitSet::new()
    }
}

impl<'a, T> SystemData<'a> for Fetch<'a, T>
where
    T: Component,
{
    fn fetch(world: &'a World) -> Self {
        Fetch {
            arena: world.arena_raw::<T>().borrow(),
        }
    }
}

impl<'a, T> SystemDataMut<'a> for Fetch<'a, T>
where
    T: Component,
{
    fn fetch_mut(world: &'a World) -> Self {
        Fetch {
            arena: world.arena_raw::<T>().borrow(),
        }
    }
}

impl<'a, T> SystemDataMask for Fetch<'a, T>
where
    T: Component,
{
    fn readables(world: &World) -> BitSet {
        let mut mask = BitSet::new();
        mask.insert(world.index::<T>());
        mask
    }

    fn writables(_: &World) -> BitSet {
        BitSet::new()
    }
}

impl<'a, T> SystemDataMut<'a> for FetchMut<'a, T>
where
    T: Component,
{
    fn fetch_mut(world: &'a World) -> Self {
        FetchMut {
            arena: world.arena_raw::<T>().borrow_mut(),
        }
    }
}

impl<'a, T> SystemDataMask for FetchMut<'a, T>
where
    T: Component,
{
    fn readables(_: &World) -> BitSet {
        BitSet::new()
    }

    fn writables(world: &World) -> BitSet {
        let mut mask = BitSet::new();
        mask.insert(world.index::<T>());
        mask
    }
}

macro_rules! impl_system_data_mut {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty), *> SystemDataMut<'a> for ( $( $ty , )* )
        where
            $( $ty : SystemDataMut<'a> ),*
        {
            fn fetch_mut(world: &'a World) -> Self {
                ( $( <$ty as SystemDataMut>::fetch_mut(world), )* )
            }
        }

        impl<'a, $($ty), *> SystemDataMask for ( $( $ty , )* )
        where
            $( $ty : SystemData<'a> + SystemDataMask ),*
        {
            fn readables(world: &World) -> BitSet {
                let mut mask = BitSet::new();

                $( {
                    mask = mask.union_with(<$ty as SystemDataMask>::readables(world));
                } )*

                mask
            }

            fn writables(world: &World) -> BitSet {
                let mut mask = BitSet::new();

                $( {
                    mask = mask.union_with(<$ty as SystemDataMask>::writables(world));
                } )*

                mask
            }
        }
    };
}

macro_rules! impl_system_data {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty), *> SystemData<'a> for ( $( $ty , )* )
        where
            $( $ty : SystemData<'a> ),*
        {
            fn fetch(world: &'a World) -> Self {
                ( $( <$ty as SystemData>::fetch(world), )* )
            }
        }
    };
}

impl_system_data!(T1);
impl_system_data_mut!(T1);
impl_system_data!(T1, T2);
impl_system_data_mut!(T1, T2);
impl_system_data!(T1, T2, T3);
impl_system_data_mut!(T1, T2, T3);
impl_system_data!(T1, T2, T3, T4);
impl_system_data_mut!(T1, T2, T3, T4);
impl_system_data!(T1, T2, T3, T4, T5);
impl_system_data_mut!(T1, T2, T3, T4, T5);
impl_system_data!(T1, T2, T3, T4, T5, T6);
impl_system_data_mut!(T1, T2, T3, T4, T5, T6);
impl_system_data!(T1, T2, T3, T4, T5, T6, T7);
impl_system_data_mut!(T1, T2, T3, T4, T5, T6, T7);
impl_system_data!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_system_data_mut!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_system_data!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_system_data_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
