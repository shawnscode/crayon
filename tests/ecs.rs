#[macro_use]
extern crate crayon;
#[macro_use]
extern crate lazy_static;
extern crate rand;

use crayon::prelude::*;
use std::sync::{Arc, RwLock};
use rand::{Rng, SeedableRng, XorShiftRng};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Position {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, Default)]
struct Reference {
    value: Arc<RwLock<usize>>,
}

impl Drop for Reference {
    fn drop(&mut self) {
        *self.value.write().unwrap() += 1;
    }
}

declare_component!(Position, VecArena);
declare_component!(Reference, HashMapArena);

#[test]
fn basic() {
    let mut world = World::new();
    world.register::<Position>();

    let e1 = world.create();
    world.add::<Position>(e1, Position { x: 1, y: 2 });
    assert!(world.has::<Position>(e1));

    {
        let p = world.get::<Position>(e1).unwrap();
        assert_eq!(*p, Position { x: 1, y: 2 });
    }

    {
        let mut p = world.get_mut::<Position>(e1).unwrap();
        p.x = 2;
        p.y = 5;
    }

    {
        let p = world.get::<Position>(e1).unwrap();
        assert_eq!(*p, Position { x: 2, y: 5 });
    }

    world.remove::<Position>(e1);
    assert!(!world.has::<Position>(e1));
    assert!(world.get::<Position>(e1).is_none());
}

#[test]
fn free() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Reference>();

    let e1 = world.create();
    assert!(world.is_alive(e1));
    assert!(!world.has::<Position>(e1));
    assert!(world.get::<Position>(e1).is_none());

    world.add::<Position>(e1, Position { x: 1, y: 2 });
    assert!(world.has::<Position>(e1));
    world.get::<Position>(e1).unwrap();

    world.free(e1);
    assert!(!world.is_alive(e1));
    assert!(!world.has::<Position>(e1));
    assert!(world.get::<Position>(e1).is_none());

    let mut entities = Vec::new();
    let rc = Arc::new(RwLock::new(0));
    for i in 0..10 {
        let e = world.create();
        let shadow = rc.clone();
        entities.push(e);

        world.add::<Reference>(e, Reference { value: shadow });
        if i % 2 == 0 {
            world.add::<Position>(e, Position { x: 1, y: 2 });
        }
    }

    assert_eq!(*rc.read().unwrap(), 0);
    for i in 0..10 {
        world.free(entities[i]);
        assert_eq!(*rc.read().unwrap(), i + 1);
    }
    assert_eq!(*rc.read().unwrap(), 10);
}

#[test]
fn duplicated_add() {
    let mut world = World::new();
    world.register::<Position>();

    let e1 = world.create();
    assert!(world.add::<Position>(e1, Position { x: 1, y: 2 }) == None);
    assert!(world.add::<Position>(e1, Position { x: 2, y: 4 }) == Some(Position { x: 1, y: 2 }));

    assert!(*world.get::<Position>(e1).unwrap() == Position { x: 2, y: 4 })
}

#[test]
fn random_allocate() {
    let mut generator = XorShiftRng::from_seed([0, 1, 2, 3]);
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Reference>();

    let mut v = vec![];
    for i in 3..10 {
        let p = generator.next_u32() % i + 1;
        let r = generator.next_u32() % i + 1;
        for j in 0..100 {
            if j % p == 0 {
                let e = world.create();
                world.add::<Position>(e,
                                      Position {
                                          x: e.index(),
                                          y: e.version(),
                                      });
                if j % r == 0 {
                    world.add_with_default::<Reference>(e);
                }
                v.push(e);
            }
        }

        let size = v.len() / 2;
        for _ in 0..size {
            let len = v.len();
            world.free(v.swap_remove(generator.next_u32() as usize % len));
        }
    }

    for i in v {
        assert_eq!(*world.get::<Position>(i).unwrap(),
                   Position {
                       x: i.index(),
                       y: i.version(),
                   });
    }
}

#[test]
fn iter_with() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Reference>();

    let mut v = vec![];
    for i in 0..100 {
        let e = world.create();

        if i % 2 == 0 {
            world.add::<Position>(e,
                                  Position {
                                      x: e.index(),
                                      y: e.version(),
                                  });
        }

        if i % 3 == 0 {
            world.add_with_default::<Reference>(e);
        }

        if i % 2 == 0 && i % 3 == 0 {
            v.push(e);
        }
    }

    {
        let (view, arenas) = world.view_with_2::<Position, Reference>();
        for e in view {
            let p = Position {
                x: e.index(),
                y: e.version(),
            };

            assert_eq!(*arenas.0.get(e).unwrap(), p);
        }
    }

    {
        let (view, mut arenas) = world.view_with_2::<Position, Reference>();
        for e in view {
            arenas.0.get_mut(e).unwrap().x += e.version();
            *arenas.1.get_mut(e).unwrap().value.write().unwrap() += 1;
        }
    }

    {
        let (view, arenas) = world.view_with_2::<Position, Reference>();
        let mut iterator = view.into_iter();
        for e in &v {
            let i = iterator.next().unwrap();
            let p = Position {
                x: e.index() + e.version(),
                y: e.version(),
            };

            assert_eq!(i, *e);
            assert_eq!(*arenas.0.get(e).unwrap(), p);
            assert_eq!(*arenas.1.get(e).unwrap().value.read().unwrap(), 1);
        }
    }
}

#[test]
#[should_panic]
fn invalid_view() {
    let mut world = World::new();
    world.register::<Position>();

    let _i1 = world.view_with::<Position>();
    world.view_with::<Position>();
}

#[test]
fn builder() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Reference>();

    let e1 = world.build().with_default::<Position>().finish();
    assert!(world.has::<Position>(e1));
    assert!(!world.has::<Reference>(e1));
}

#[test]
#[should_panic]
fn invalid_get() {
    let mut world = World::new();
    world.register::<Position>();

    let e1 = world.build().with_default::<Position>().finish();

    let _p1 = world.get_mut::<Position>(e1);
    world.get::<Position>(e1);
}

#[test]
#[should_panic]
fn invalid_get_mut() {
    let mut world = World::new();
    world.register::<Position>();

    let e1 = world.build().with_default::<Position>().finish();

    let _p1 = world.get_mut::<Position>(e1);
    world.get_mut::<Position>(e1);
}