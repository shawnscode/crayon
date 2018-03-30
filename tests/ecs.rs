extern crate crayon;
extern crate rand;

use crayon::ecs::prelude::*;

use std::sync::{Arc, RwLock};
use rand::{Rng, SeedableRng, XorShiftRng};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
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

impl Component for Position {
    type Arena = VecArena<Position>;
}

impl Component for Reference {
    type Arena = HashMapArena<Reference>;
}

// struct IncXSystem {}
// struct DecXSystem {}

// impl<'a> SystemMut<'a> for IncXSystem {
//     type ViewWithMut = FetchMut<'a, Position>;
//     type ResultMut = ();

//     fn run_mut(&mut self, view: View, mut arena: Self::ViewWithMut) {
//         unsafe {
//             for v in view {
//                 arena.get_unchecked_mut(v).x += 1;
//             }
//         }
//     }
// }

// impl<'a> SystemMut<'a> for DecXSystem {
//     type ViewWithMut = FetchMut<'a, Position>;
//     type ResultMut = ();

//     fn run_mut(&mut self, view: View, mut arena: Self::ViewWithMut) {
//         unsafe {
//             for v in view {
//                 arena.get_unchecked_mut(v).x -= 1;
//             }
//         }
//     }
// }

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
        let (_, mut arena) = world.view_w1::<Position>();

        let p = arena.get_mut(e1).unwrap();
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
#[should_panic]
fn storage_not_registered() {
    let mut world = World::new();

    let e1 = world.create();
    world.add::<Position>(e1, Position { x: 1, y: 2 });
}

#[test]
fn get() {
    let mut world = World::new();
    world.register::<Position>();

    let e1 = world.create();
    world.add::<Position>(e1, Position { x: 1, y: 2 });
    assert!(world.has::<Position>(e1));
    assert!(world.get::<Position>(e1).is_some());

    *world.get_mut(e1).unwrap() = Position { x: 2, y: 3 };
    assert_eq!(*world.get::<Position>(e1).unwrap(), Position { x: 2, y: 3 });

    {
        let _p1 = world.get::<Position>(e1);
        let _p2 = world.get::<Position>(e1);
    }
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
    for (i, e) in entities.iter().enumerate().take(10) {
        world.free(*e);
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
                world.add::<Position>(
                    e,
                    Position {
                        x: e.index(),
                        y: e.version(),
                    },
                );
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
        assert_eq!(
            *world.get::<Position>(i).unwrap(),
            Position {
                x: i.index(),
                y: i.version(),
            }
        );
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
            world.add::<Position>(
                e,
                Position {
                    x: e.index(),
                    y: e.version(),
                },
            );
        }

        if i % 3 == 0 {
            world.add_with_default::<Reference>(e);
        }

        if i % 2 == 0 && i % 3 == 0 {
            v.push(e);
        }
    }

    {
        let (entities, a1, a2) = world.view_r2::<Position, Reference>();
        assert_eq!((&a1, &a2).join(&entities).count(), v.len());
    }

    {
        let (entities, a1, a2) = world.view_r2::<Position, Reference>();

        for (e, position, _) in (entities, &a1, &a2).join(&entities) {
            let p = Position {
                x: e.index(),
                y: e.version(),
            };

            assert_eq!(*position, p);
        }
    }

    {
        let (entities, mut a1, mut a2) = world.view_w2::<Position, Reference>();
        for (e, mut position, mut reference) in (entities, &mut a1, &mut a2).join(&entities) {
            position.x += e.version();
            *reference.value.write().unwrap() += 1;
        }
    }

    {
        let (entities, a1, a2) = world.view_r2::<Position, Reference>();
        let mut iter = (entities, &a1, &a2).join(&entities);
        for e in &v {
            let (i, position, reference) = iter.next().unwrap();
            let p = Position {
                x: e.index() + e.version(),
                y: e.version(),
            };

            assert_eq!(i, *e);
            assert_eq!(*position, p);
            assert_eq!(*reference.value.read().unwrap(), 1);
        }
    }
}

#[test]
#[should_panic]
fn storage_already_borrowed_mutably() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Reference>();

    let _ = world.view_r1w1::<Position, Position>();
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

// #[test]
// fn system() {
//     let mut world = World::new();
//     world.register::<Position>();
//     let e1 = world.build().with_default::<Position>().finish();

//     let mut inc = IncXSystem {};
//     inc.run_mut_at(&mut world);
//     assert!(world.get::<Position>(e1).unwrap().x == 1);

//     let mut dec = DecXSystem {};
//     dec.run_mut_at(&mut world);
//     assert!(world.get::<Position>(e1).unwrap().x == 0);

//     // assert!(!validate(&world, &[&inc, &dec]));
// }
