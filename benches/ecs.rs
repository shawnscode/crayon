#![feature(test)]
extern crate crayon;
extern crate rayon;
extern crate test;

use test::Bencher;
use crayon::prelude::*;

use std::thread;
use std::time;

fn execute() {
    thread::sleep(time::Duration::from_millis(1));
}

fn setup() -> World {
    let mut world = World::new();
    world.register::<PlaceHolderCMP>();

    for _ in 1..3 {
        world.build().with_default::<PlaceHolderCMP>();
    }

    world
}

#[derive(Debug, Copy, Clone, Default)]
struct PlaceHolderCMP {}

impl Component for PlaceHolderCMP {
    type Arena = ecs::VecArena<PlaceHolderCMP>;
}

#[derive(Copy, Clone)]
struct HeavyCPU {}

impl<'a> System<'a> for HeavyCPU {
    type ViewWith = Fetch<'a, PlaceHolderCMP>;
    type Result = ();

    fn run(&self, view: View, _: Self::ViewWith) {
        for _ in view {
            execute();
        }
    }
}

#[derive(Copy, Clone)]
struct HeavyCPUWithRayon {}

impl<'a> System<'a> for HeavyCPUWithRayon {
    type ViewWith = Fetch<'a, PlaceHolderCMP>;
    type Result = ();

    fn run(&self, view: View, _: Self::ViewWith) {
        rayon::scope(|s| {
            //
            for _ in view {
                s.spawn(|_| execute());
            }
        })
    }
}

#[bench]
fn bench_sequence_execution(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let s1 = HeavyCPU {};
        let s2 = HeavyCPU {};
        let s3 = HeavyCPU {};
        s1.run_at(&world);
        s2.run_at(&world);
        s3.run_at(&world);
    });
}

#[bench]
fn bench_parralle_execution(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let s1 = HeavyCPU {};
        let s2 = HeavyCPU {};
        let s3 = HeavyCPU {};

        rayon::scope(|s| {
            //
            s.spawn(|_| s1.run_at(&world));
            s.spawn(|_| s2.run_at(&world));
            s.spawn(|_| s3.run_at(&world));
        });
    });
}

#[bench]
fn bench_parralle_execution_2(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let s1 = HeavyCPUWithRayon {};
        let s2 = HeavyCPUWithRayon {};
        let s3 = HeavyCPUWithRayon {};

        rayon::scope(|s| {
            //
            s.spawn(|_| s1.run_at(&world));
            s.spawn(|_| s2.run_at(&world));
            s.spawn(|_| s3.run_at(&world));
        });
    });
}
