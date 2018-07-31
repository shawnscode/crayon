//! Provides batch exection operations based on `World` in concurrent environments.

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use failure;

use ecs::bitset::BitSet;
use ecs::component::Component;
use ecs::view::{Fetch, FetchMut};
use ecs::world::{Entities, World};
use sched::ScheduleSystemShared;
use utils::{Handle, HandlePool};

pub type Result<E> = ::std::result::Result<(), E>;

/// A `System` could be executed with a set of required `SystemData`s.
pub trait System<'w> {
    type Data: SystemData<'w>;
    type Err: failure::Fail;

    fn run(&mut self, _: Self::Data) -> Result<Self::Err>;

    /// Runs system with given `World`.
    ///
    /// # Panics
    ///
    /// Panics if the system is trying to access resource mutably.
    fn run_with(&mut self, world: &'w World) -> Result<Self::Err> {
        unsafe {
            assert!(
                Self::Data::writables(world).is_empty(),
                "Can't run this system with immutable world."
            );

            self.run(Self::Data::fetch(world))
        }
    }

    /// Runs system with given mutable `World`.
    fn run_with_mut(&mut self, world: &'w mut World) -> Result<Self::Err> {
        unsafe { self.run(Self::Data::fetch(world)) }
    }
}

/// The execution dispatcher of systems.
///
/// Notes that the decision of whether or not to execute systems in paralle is made
/// dynamically, based on their dependencies and resource accessing requests.
pub struct SystemDispatcher<'scope, E: failure::Fail> {
    handles: HandlePool,
    systems: Vec<Option<SystemDispatcherItem<'scope, E>>>,
}

struct SystemDispatcherItem<'scope, E: failure::Fail> {
    system: Box<for<'w> Executable<'w, Err = E> + Send + 'scope>,
    children: Vec<Handle>,
    dependencies: Vec<Handle>,
}

impl<'scope, E: failure::Fail> SystemDispatcher<'scope, E> {
    pub fn new() -> Self {
        SystemDispatcher {
            handles: HandlePool::new(),
            systems: Vec::new(),
        }
    }

    /// Adds a new system with a list of dependencies. The dependents will always be executed
    /// before the execution of this system.
    ///
    /// Returns a `Handle` as the unique identifier of this system.
    pub fn add<T>(&mut self, dependencies: &[Handle], system: T) -> Handle
    where
        T: for<'w> System<'w, Err = E> + Send + 'scope,
    {
        self.add_executable(dependencies, system)
    }

    /// Dispatches the systems in parallel given the resources to operate on.
    ///
    /// This operation will blocks the current thread until we finished all the executions. First
    /// error that produces during execution will be returned.
    pub fn run(&mut self, world: &mut World, sched: &ScheduleSystemShared) -> Result<E> {
        unsafe {
            let batches = self.build(world);

            for mut v in batches {
                sched.scope(|s| {
                    let world = &world;
                    for mut w in v {
                        s.spawn(move |_| {
                            w.execute(world).unwrap();
                        });
                    }
                });
            }

            Ok(())
        }
    }

    /// Dispatches the systems sequentially given the resources to operate on.
    ///
    /// This operation will blocks the current thread until we finished all the executions. First
    /// error that produces during execution will be returned.
    pub fn run_in_sequence(&mut self, world: &mut World) -> Result<E> {
        unsafe {
            for k in &self.handles {
                let item = self.systems
                    .get_unchecked_mut(k.index() as usize)
                    .as_mut()
                    .unwrap();

                item.system.execute(world)?;
            }

            self.handles.clear();
            self.systems.clear();
            Ok(())
        }
    }

    unsafe fn build(
        &mut self,
        world: &mut World,
    ) -> Vec<Vec<Box<Executable<Err = E> + Send + 'scope>>> {
        let mut dependents = HashMap::new();
        for k in &self.handles {
            let item = self.systems
                .get_unchecked(k.index() as usize)
                .as_ref()
                .unwrap();
            dependents.insert(k, item.dependencies.len());
        }

        let mut results = Vec::new();
        let mut fbuf = Vec::new();
        let mut bbuf = Vec::new();
        let mut batch_readables = BitSet::new();
        let mut batch_writables = BitSet::new();
        while !dependents.is_empty() {
            ::std::mem::swap(&mut fbuf, &mut bbuf);

            dependents.retain(|k, v| {
                if *v == 0 {
                    fbuf.push(*k);
                    false
                } else {
                    true
                }
            });

            fbuf.sort_by(|a, b| {
                let a1 = self.systems
                    .get_unchecked(a.index() as usize)
                    .as_ref()
                    .unwrap()
                    .children
                    .len();

                let b1 = self.systems
                    .get_unchecked(b.index() as usize)
                    .as_ref()
                    .unwrap()
                    .children
                    .len();

                b1.cmp(&a1)
            });

            let mut batch = Vec::new();
            batch_readables.clear();
            batch_writables.clear();
            bbuf.clear();
            for handle in fbuf.iter_mut() {
                let (readables, writables) = {
                    let item = self.systems
                        .get_unchecked(handle.index() as usize)
                        .as_ref()
                        .unwrap();

                    let readables = item.system.readables(world);
                    let writables = item.system.writables(world);
                    (readables, writables)
                };

                let can_write = batch_readables.is_disjoint(writables);
                let can_read = batch_writables.is_disjoint(readables);
                if can_write && can_read {
                    let item = self.systems
                        .get_unchecked_mut(handle.index() as usize)
                        .take()
                        .unwrap();

                    for child in &item.children {
                        if let Some(degree) = dependents.get_mut(child) {
                            *degree -= 1;
                        }
                    }

                    batch_readables = batch_readables.union_with(readables);
                    batch_writables = batch_writables.union_with(writables);
                    batch.push(item.system);
                } else {
                    bbuf.push(*handle);
                }
            }

            if batch.is_empty() {
                break;
            }

            results.push(batch);
        }

        assert!(dependents.len() == 0);
        self.handles.clear();
        self.systems.clear();
        results
    }

    fn add_executable<T1>(&mut self, dependencies: &[Handle], system: T1) -> Handle
    where
        T1: for<'w> Executable<'w, Err = E> + Send + 'scope,
    {
        let deps: HashSet<Handle> = dependencies
            .iter()
            .cloned()
            .filter(|v| self.handles.is_alive(v))
            .collect();

        let handle = self.handles.create();
        let item = SystemDispatcherItem {
            dependencies: Vec::from_iter(deps.iter().cloned()),
            children: Vec::new(),
            system: Box::new(system),
        };

        if self.systems.len() <= (handle.index() as usize) {
            self.systems.push(Some(item));
        } else {
            self.systems[handle.index() as usize] = Some(item);
        }

        unsafe {
            for v in deps {
                let mut item = self.systems
                    .get_unchecked_mut(v.index() as usize)
                    .as_mut()
                    .unwrap();
                item.children.push(handle);
            }
        }

        handle
    }
}

macro_rules! impl_run_closure {
    ($name: ident, [$($readables: ident),*], [$($writables: ident),*]) => (
        impl<'scope, E: failure::Fail> SystemDispatcher<'scope, E> {
            /// Queues a new system into the command buffer.
            ///
            /// Each system queued within a single `SystemDispatcher` may be executed
            /// in parallel with each other.
            #[allow(non_snake_case, unused_variables, unused_mut)]
            pub fn $name<$($readables,)* $($writables,)* F>(&mut self, dependencies: &[Handle], mut f: F) -> Handle
            where
                $($readables: Component,)*
                $($writables: Component,)*
                F: for<'a> FnMut(Entities<'a>, $(Fetch<'a, $readables>,)* $(FetchMut<'a, $writables>,)*) -> Result<E> + Send + 'scope
            {
                let closure = move |world: &World| {
                    // Safety of these fetches is ensured by the system scheduler.
                    unsafe {
                        $(let $readables = Fetch::new(world);)*
                        $(let $writables = FetchMut::new(world);)*
                        f(world.entities(), $($readables,)* $($writables,)*)
                    }
                };

                let readables = move |world: &World| -> BitSet {
                    BitSet::from(&[$(world.mask_index::<$readables>(),)*])
                };

                let writables = move |world: &World| -> BitSet {
                    BitSet::from(&[$(world.mask_index::<$writables>(),)*])
                };

                self.add_executable(dependencies, ClosureExecutable {
                    closure: closure,
                    readables: readables,
                    writables: writables,
                })
            }
        }
    )
}

impl_run_closure!(add_r1, [R1], []);
impl_run_closure!(add_r2, [R1, R2], []);
impl_run_closure!(add_r3, [R1, R2, R3], []);
impl_run_closure!(add_r4, [R1, R2, R3, R4], []);
impl_run_closure!(add_r5, [R1, R2, R3, R4, R5], []);
impl_run_closure!(add_r6, [R1, R2, R3, R4, R5, R6], []);
impl_run_closure!(add_r7, [R1, R2, R3, R4, R5, R6, R7], []);
impl_run_closure!(add_r8, [R1, R2, R3, R4, R5, R6, R7, R8], []);
impl_run_closure!(add_r9, [R1, R2, R3, R4, R5, R6, R7, R8, R9], []);

impl_run_closure!(add_w1, [], [W1]);
impl_run_closure!(add_w2, [], [W1, W2]);
impl_run_closure!(add_w3, [], [W1, W2, W3]);
impl_run_closure!(add_w4, [], [W1, W2, W3, W4]);
impl_run_closure!(add_w5, [], [W1, W2, W3, W4, W5]);
impl_run_closure!(add_w6, [], [W1, W2, W3, W4, W5, W6]);
impl_run_closure!(add_w7, [], [W1, W2, W3, W4, W5, W6, W7]);
impl_run_closure!(add_w8, [], [W1, W2, W3, W4, W5, W6, W7, W8]);
impl_run_closure!(add_w9, [], [W1, W2, W3, W4, W5, W6, W7, W8, W9]);

impl_run_closure!(add_r1w1, [T1], [W1]);
impl_run_closure!(add_r1w2, [T1], [W1, W2]);
impl_run_closure!(add_r1w3, [T1], [W1, W2, W3]);
impl_run_closure!(add_r1w4, [T1], [W1, W2, W3, W4]);

impl_run_closure!(add_r2w1, [T1, T2], [W1]);
impl_run_closure!(add_r2w2, [T1, T2], [W1, W2]);
impl_run_closure!(add_r2w3, [T1, T2], [W1, W2, W3]);
impl_run_closure!(add_r2w4, [T1, T2], [W1, W2, W3, W4]);

impl_run_closure!(add_r3w1, [T1, T2, T3], [W1]);
impl_run_closure!(add_r3w2, [T1, T2, T3], [W1, W2]);
impl_run_closure!(add_r3w3, [T1, T2, T3], [W1, W2, W3]);
impl_run_closure!(add_r3w4, [T1, T2, T3], [W1, W2, W3, W4]);

impl_run_closure!(add_r4w1, [T1, T2, T3, T4], [W1]);
impl_run_closure!(add_r4w2, [T1, T2, T3, T4], [W1, W2]);
impl_run_closure!(add_r4w3, [T1, T2, T3, T4], [W1, W2, W3]);
impl_run_closure!(add_r4w4, [T1, T2, T3, T4], [W1, W2, W3, W4]);

trait Executable<'w> {
    type Err: failure::Fail + Send;

    fn execute(&mut self, _: &'w World) -> Result<Self::Err>;
    fn readables(&self, _: &World) -> BitSet;
    fn writables(&self, _: &World) -> BitSet;
}

struct ClosureExecutable<F1, F2, F3, E>
where
    F1: FnMut(&World) -> Result<E> + Send,
    F2: Fn(&World) -> BitSet,
    F3: Fn(&World) -> BitSet,
    E: Sized + Send,
{
    closure: F1,
    readables: F2,
    writables: F3,
}

impl<'w, F1, F2, F3, E> Executable<'w> for ClosureExecutable<F1, F2, F3, E>
where
    F1: FnMut(&World) -> Result<E> + Send,
    F2: Fn(&World) -> BitSet,
    F3: Fn(&World) -> BitSet,
    E: failure::Fail,
{
    type Err = E;

    fn execute(&mut self, world: &'w World) -> Result<Self::Err> {
        (self.closure)(world)
    }

    fn readables(&self, world: &World) -> BitSet {
        (self.readables)(world)
    }

    fn writables(&self, world: &World) -> BitSet {
        (self.writables)(world)
    }
}

impl<'w, T> Executable<'w> for T
where
    T: System<'w>,
{
    type Err = <Self as System<'w>>::Err;

    fn execute(&mut self, world: &'w World) -> Result<Self::Err> {
        unsafe { self.run(T::Data::fetch(world)) }
    }

    fn readables(&self, world: &World) -> BitSet {
        unsafe { T::Data::readables(world) }
    }

    fn writables(&self, world: &World) -> BitSet {
        unsafe { T::Data::writables(world) }
    }
}

/// A `SystemData` addresses a set of resources which are required for the execution
/// of some kind of `System`.
pub trait SystemData<'s> {
    #[doc(hidden)]
    unsafe fn fetch(world: &'s World) -> Self;

    #[doc(hidden)]
    unsafe fn readables(world: &World) -> BitSet;

    #[doc(hidden)]
    unsafe fn writables(world: &World) -> BitSet;
}

impl<'w, T: Component> SystemData<'w> for Fetch<'w, T> {
    unsafe fn fetch(world: &'w World) -> Self {
        Fetch::new(world)
    }

    unsafe fn readables(world: &World) -> BitSet {
        BitSet::from(&[world.mask_index::<T>()])
    }

    unsafe fn writables(_: &World) -> BitSet {
        BitSet::new()
    }
}

impl<'w, T: Component> SystemData<'w> for FetchMut<'w, T> {
    unsafe fn fetch(world: &'w World) -> Self {
        FetchMut::new(world)
    }

    unsafe fn readables(_: &World) -> BitSet {
        BitSet::new()
    }

    unsafe fn writables(world: &World) -> BitSet {
        BitSet::from(&[world.mask_index::<T>()])
    }
}

macro_rules! impl_system_data {
    ([$($tps: ident),*]) => (
        impl<'w, $($tps: SystemData<'w>,)*> SystemData<'w> for ($($tps,)*) {
            unsafe fn fetch(world: &'w World) -> Self {
                ( $($tps::fetch(world),)* )
            }

            unsafe fn readables(world: &World) -> BitSet {
                let mut bits = BitSet::new();
                $( bits = bits.union_with($tps::readables(world)); )*
                bits
            }

            unsafe fn writables(world: &World) -> BitSet {
                let mut bits = BitSet::new();
                $( bits = bits.union_with($tps::writables(world)); )*
                bits
            }
        }
    )
}

impl_system_data!([T1]);
impl_system_data!([T1, T2]);
impl_system_data!([T1, T2, T3]);
impl_system_data!([T1, T2, T3, T4]);
impl_system_data!([T1, T2, T3, T4, T5]);
impl_system_data!([T1, T2, T3, T4, T5, T6]);
impl_system_data!([T1, T2, T3, T4, T5, T6, T7]);
impl_system_data!([T1, T2, T3, T4, T5, T6, T7, T8]);
impl_system_data!([T1, T2, T3, T4, T5, T6, T7, T8, T9]);

#[cfg(test)]
mod test {
    use super::super::component::VecArena;
    use super::super::view::Join;
    use super::*;
    use sched::ScheduleSystem;

    struct Value {
        value: i32,
    }

    impl Component for Value {
        type Arena = VecArena<Value>;
    }

    struct OtherValue {}
    impl Component for OtherValue {
        type Arena = VecArena<OtherValue>;
    }

    struct OtherValue2 {}
    impl Component for OtherValue2 {
        type Arena = VecArena<OtherValue2>;
    }

    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "None")]
        _None,
    }

    struct ValueSystem {}
    impl<'w> System<'w> for ValueSystem {
        type Data = FetchMut<'w, Value>;
        type Err = Error;

        fn run(&mut self, data: Self::Data) -> Result<Error> {
            for mut v in data.join() {
                v.value += 1;
            }

            Ok(())
        }
    }

    struct OtherSystem {}
    impl<'w> System<'w> for OtherSystem {
        type Data = Fetch<'w, Value>;
        type Err = Error;

        fn run(&mut self, _: Self::Data) -> Result<Error> {
            Ok(())
        }
    }

    struct OtherSystem2 {}
    impl<'w> System<'w> for OtherSystem2 {
        type Data = FetchMut<'w, OtherValue>;
        type Err = Error;

        fn run(&mut self, _: Self::Data) -> Result<Error> {
            Ok(())
        }
    }

    struct OtherSystem3 {}
    impl<'w> System<'w> for OtherSystem3 {
        type Data = Fetch<'w, OtherValue2>;
        type Err = Error;

        fn run(&mut self, _: Self::Data) -> Result<Error> {
            Ok(())
        }
    }

    #[test]
    fn batch() {
        let sched = ScheduleSystem::new(4, None, None);
        let mut world = World::new();
        world.register::<Value>();
        world.register::<OtherValue>();
        world.register::<OtherValue2>();

        let ent = world.build().with(Value { value: 0 }).finish();

        let mut batch = SystemDispatcher::new();

        {
            let handle_a = batch.add_r1(&[], |_: Entities, _: Fetch<Value>| Ok(()));

            batch.add_w1(&[], |_: Entities, _: FetchMut<Value>| Ok(()));
            batch.add_w1(&[], |_: Entities, _: FetchMut<OtherValue>| Ok(()));
            batch.add_r1(&[handle_a], |_: Entities, _: Fetch<OtherValue2>| Ok(()));

            unsafe {
                let batches = batch.build(&mut world);
                assert!(batches.len() == 2);
                assert!(batches[0].len() == 2);
                assert!(batches[1].len() == 2);
            }
        }

        {
            let handle_a = batch.add_r1(&[], |_: Entities, _: Fetch<Value>| Ok(()));
            batch.add_w1(&[], |_: Entities, a: FetchMut<Value>| {
                for mut v in a.join() {
                    v.value += 1;
                }

                Ok(())
            });

            batch.add_w1(&[], |_: Entities, _: FetchMut<OtherValue>| Ok(()));
            batch.add_r1(&[handle_a], |_: Entities, _: Fetch<OtherValue2>| Ok(()));
            batch.run(&mut world, &sched.shared()).unwrap();

            assert_eq!(world.get::<Value>(ent).unwrap().value, 1);
        }

        {
            let handle_a = batch.add(&[], ValueSystem {});
            batch.add(&[], OtherSystem {});
            batch.add(&[], OtherSystem2 {});
            batch.add(&[handle_a], OtherSystem3 {});

            unsafe {
                let batches = batch.build(&mut world);
                assert!(batches.len() == 2);
                assert!(batches[0].len() == 2);
                assert!(batches[1].len() == 2);
            }
        }

        {
            let handle_a = batch.add(&[], OtherSystem {});
            batch.add(&[], ValueSystem {});
            batch.add(&[], OtherSystem2 {});
            batch.add(&[handle_a], OtherSystem3 {});
            batch.run(&mut world, &sched.shared()).unwrap();

            assert_eq!(world.get::<Value>(ent).unwrap().value, 2);
        }
    }
}
