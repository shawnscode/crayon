//! Execution utilities based on `View` and `Arena`s.

use super::{World, View, Fetch, FetchMut, Component};
use super::bitset::BitSet;

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
pub trait System<'a> {
    /// The component arenas required to execute system.
    type ViewWith: SystemData<'a>;

    /// Run the system with the required arenas.
    fn run(&mut self, view: View, arenas: Self::ViewWith);

    fn run_at(&mut self, world: &'a World) {
        let r = Self::ViewWith::readables(world);
        let w = Self::ViewWith::writables(world);
        let mask = r.union_with(w);

        self.run(world.view(mask), Self::ViewWith::fetch(world));
    }
}

/// Trait for validation system.
pub trait SystemValidator {
    fn readables(&self, world: &World) -> BitSet;
    fn writables(&self, world: &World) -> BitSet;
}

impl<'a, T> SystemValidator for T
    where T: System<'a>
{
    fn readables(&self, world: &World) -> BitSet {
        T::ViewWith::readables(world)
    }

    fn writables(&self, world: &World) -> BitSet {
        T::ViewWith::writables(world)
    }
}

/// Returns true if the systems could run at the same time safely.
pub fn validate<'a>(world: &'a World, systems: &[&SystemValidator]) -> bool {
    let mut r = BitSet::new();
    let mut w = BitSet::new();

    for s in systems {
        r = r.union_with(s.readables(world));

        if !w.intersect_with(s.writables(world)).is_empty() {
            return false;
        }

        w = w.union_with(s.writables(world));
    }

    w.intersect_with(r).is_empty()
}

/// A struct implementing `SystemData` indicates that it bundles some arenas which
/// are required for execution.
pub trait SystemData<'a> {
    /// Creates a new arena bundle by fetching from `World`.
    fn fetch(world: &'a World) -> Self;
    /// Gets the mask of readable component arenas.
    fn readables(world: &World) -> BitSet;
    /// Gets the mask of writable component arenas.
    fn writables(world: &World) -> BitSet;
}

impl<'a> SystemData<'a> for () {
    fn fetch(_: &'a World) -> Self {
        ()
    }

    fn readables(_: &World) -> BitSet {
        BitSet::new()
    }

    fn writables(_: &World) -> BitSet {
        BitSet::new()
    }
}

impl<'a, T> SystemData<'a> for Fetch<'a, T>
    where T: Component
{
    fn fetch(world: &'a World) -> Self {
        world.arena::<T>()
    }

    fn readables(world: &World) -> BitSet {
        let mut mask = BitSet::new();
        mask.insert(world.index::<T>());
        mask
    }

    fn writables(_: &World) -> BitSet {
        BitSet::new()
    }
}

impl<'a, T> SystemData<'a> for FetchMut<'a, T>
    where T: Component
{
    fn fetch(world: &'a World) -> Self {
        world.arena_mut::<T>()
    }

    fn readables(_: &World) -> BitSet {
        BitSet::new()
    }

    fn writables(world: &World) -> BitSet {
        let mut mask = BitSet::new();
        mask.insert(world.index::<T>());
        mask
    }
}

macro_rules! impl_system_data {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty),*> SystemData<'a> for ( $( $ty , )* )
            where $( $ty : SystemData<'a> ),*
        {
            fn fetch(world: &'a World) -> Self {
                ( $( <$ty as SystemData>::fetch(world), )* )
            }

            fn readables(world: &World) -> BitSet {
                let mut mask = BitSet::new();

                $( {
                    mask = mask.union_with(<$ty as SystemData>::readables(world));
                } )*

                mask
            }

            fn writables(world: &World) -> BitSet {
                let mut mask = BitSet::new();

                $( {
                    mask = mask.union_with(<$ty as SystemData>::writables(world));
                } )*

                mask
            }
        }
    };
}

impl_system_data!(T1);
impl_system_data!(T1, T2);
impl_system_data!(T1, T2, T3);
impl_system_data!(T1, T2, T3, T4);
impl_system_data!(T1, T2, T3, T4, T5);
impl_system_data!(T1, T2, T3, T4, T5, T6);
impl_system_data!(T1, T2, T3, T4, T5, T6, T7);
impl_system_data!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_system_data!(T1, T2, T3, T4, T5, T6, T7, T8, T9);