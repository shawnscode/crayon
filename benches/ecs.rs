#![feature(test)]
extern crate crayon;
#[macro_use]
extern crate failure;
extern crate test;

use test::Bencher;
use crayon::ecs::prelude::*;

use std::thread;
use std::time;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "None")] _None,
}

pub type Result<E> = ::std::result::Result<(), E>;

fn execute() {
    thread::sleep(time::Duration::new(0, 1000));
}

fn setup() -> World {
    let mut world = World::new();
    world.register::<Value>();

    for _ in 1..20 {
        world.build().with_default::<Value>();
    }

    world
}

#[derive(Debug, Copy, Clone, Default)]
struct Value {}

impl Component for Value {
    type Arena = VecArena<Value>;
}

struct Execution {}

impl<'a> System<'a> for Execution {
    type Data = Fetch<'a, Value>;
    type Err = Error;

    fn run(&mut self, entities: Entities, values: Self::Data) -> Result<Self::Err> {
        for _ in values.join(&entities) {
            execute();
        }

        Ok(())
    }
}

struct ParExecution {}

impl<'a> System<'a> for ParExecution {
    type Data = Fetch<'a, Value>;
    type Err = Error;

    fn run(&mut self, entities: Entities, values: Self::Data) -> Result<Self::Err> {
        values.par_join(&entities, 3).map(|_| execute()).count();
        Ok(())
    }
}

#[bench]
fn seq_execution(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let mut s1 = Execution {};
        let mut s2 = Execution {};
        let mut s3 = Execution {};
        s1.run_with(&world).unwrap();
        s2.run_with(&world).unwrap();
        s3.run_with(&world).unwrap();
    });
}

#[bench]
fn par_execution(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let mut s1 = ParExecution {};
        let mut s2 = ParExecution {};
        let mut s3 = ParExecution {};
        s1.run_with(&world).unwrap();
        s2.run_with(&world).unwrap();
        s3.run_with(&world).unwrap();
    });
}

#[bench]
fn batch_par_execution(b: &mut Bencher) {
    b.iter(|| {
        let mut world = setup();
        let mut dispatcher = SystemDispatcher::new();
        dispatcher.add(&[], ParExecution {});
        dispatcher.add(&[], ParExecution {});
        dispatcher.add(&[], ParExecution {});
        dispatcher.run(&mut world).unwrap();
    });
}
