#![feature(test)]
extern crate test;
extern crate crayon;
extern crate rayon;

use test::Bencher;
use crayon::prelude::*;

use std::thread;
use std::time;

#[derive(Debug, Copy, Clone, Default)]
struct PlaceHolderCMP {}

impl Component for PlaceHolderCMP {
    type Arena = ecs::VecArena<PlaceHolderCMP>;
}

fn execute() {
    thread::sleep(time::Duration::from_millis(1));
}

#[derive(Copy, Clone)]
struct HeavyCPU {}

impl<'a> System<'a> for HeavyCPU {
    type ViewWith = Fetch<'a, PlaceHolderCMP>;

    fn run(&mut self, view: View, _: Self::ViewWith) {
        for _ in view {
            execute();
        }
    }
}

fn setup() -> World {
    let mut world = World::new();
    world.register::<PlaceHolderCMP>();

    for _ in 1..3 {
        world.build().with_default::<PlaceHolderCMP>();
    }

    world
}

#[bench]
fn bench_sequence_execution(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let mut s1 = HeavyCPU {};
        let mut s2 = HeavyCPU {};
        let mut s3 = HeavyCPU {};
        s1.run_at(&world);
        s2.run_at(&world);
        s3.run_at(&world);
    });
}

#[bench]
fn bench_parralle_execution(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let mut s1 = HeavyCPU {};
        let mut s2 = HeavyCPU {};
        let mut s3 = HeavyCPU {};

        rayon::scope(|s| {
                         //
                         s.spawn(|_| s1.run_at(&world));
                         s.spawn(|_| s2.run_at(&world));
                         s.spawn(|_| s3.run_at(&world));
                     });
    });
}

#[derive(Copy, Clone)]
struct HeavyCPUWithRayon {}

impl<'a> System<'a> for HeavyCPUWithRayon {
    type ViewWith = Fetch<'a, PlaceHolderCMP>;

    fn run(&mut self, view: View, _: Self::ViewWith) {
        rayon::scope(|s| {
                         //
                         for _ in view {
                             s.spawn(|_| execute());
                         }
                     })
    }
}

#[bench]
fn bench_parralle_execution_2(b: &mut Bencher) {
    b.iter(|| {
        let world = setup();
        let mut s1 = HeavyCPUWithRayon {};
        let mut s2 = HeavyCPUWithRayon {};
        let mut s3 = HeavyCPUWithRayon {};

        rayon::scope(|s| {
                         //
                         s.spawn(|_| s1.run_at(&world));
                         s.spawn(|_| s2.run_at(&world));
                         s.spawn(|_| s3.run_at(&world));
                     });
    });
}
