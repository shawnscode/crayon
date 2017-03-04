extern crate lemon3d;
extern crate cgmath;
#[macro_use]
extern crate approx;
extern crate rand;

use cgmath as math;
use lemon3d::scene::transform::*;
use lemon3d::ecs::World;
use math::Zero;
use rand::{Rng, SeedableRng, XorShiftRng};

#[test]
pub fn space() {
    let mut world = World::new();
    world.register::<Transform>();

    let e1 = world.build().with_default::<Transform>().finish();
    let e2 = world.build().with_default::<Transform>().finish();
    let e3 = world.build().with_default::<Transform>().finish();
    let e4 = world.build().with_default::<Transform>().finish();

    {
        let mut arena = world.arena_mut::<Transform>().unwrap();
        Transform::set_parent(&mut arena, e4, Some(e3), false).unwrap();
        Transform::set_parent(&mut arena, e3, Some(e1), false).unwrap();
        Transform::set_parent(&mut arena, e2, Some(e1), false).unwrap();
        // e1 <- (e2, e3 <- (e4))

        Transform::set_world_position(&mut arena, e3, math::Vector3::new(1.0, 0.0, 0.0)).unwrap();
        let disp = Transform::world_position(&mut arena, e4).unwrap();
        assert_eq!(disp, math::Vector3::new(1.0, 0.0, 0.0));

        Transform::set_world_position(&mut arena, e1, math::Vector3::new(1.0, 0.0, 2.0)).unwrap();
        let disp = Transform::world_position(&mut arena, e4).unwrap();
        assert_eq!(disp, math::Vector3::new(2.0, 0.0, 2.0));

        assert_eq!(arena.get(*e4).unwrap().position(), math::Vector3::zero());
        Transform::set_parent(&mut arena, e4, Some(e2), true).unwrap();

        let disp = Transform::world_position(&mut arena, e4).unwrap();
        assert_eq!(disp, math::Vector3::new(2.0, 0.0, 2.0));
        assert_eq!(arena.get(*e4).unwrap().position(),
                   math::Vector3::new(1.0, 0.0, 0.0));

        Transform::set_world_scale(&mut arena, e4, 2.0).unwrap();
        let euler = math::Euler::new(math::Deg(0.0), math::Deg(0.0), math::Deg(90.0));
        Transform::set_world_rotation(&mut arena, e4, math::Quaternion::from(euler)).unwrap();

        let dir = Transform::transform_direction(&arena, e4, math::Vector3::new(1.0, 0.0, 0.0))
            .unwrap();
        assert!(ulps_eq!(dir, math::Vector3::new(0.0, 1.0, 0.0)));

        let vec = Transform::transform_vector(&arena, e4, math::Vector3::new(1.0, 0.0, 0.0))
            .unwrap();
        assert!(ulps_eq!(vec, math::Vector3::new(0.0, 2.0, 0.0)));

        let pos = Transform::transform_point(&arena, e4, math::Vector3::new(1.0, 0.0, 0.0))
            .unwrap();
        assert!(ulps_eq!(pos, math::Vector3::new(1.0, 4.0, 4.0)));
    }
}

#[test]
pub fn hierachy() {
    let mut world = World::new();
    world.register::<Transform>();

    let e1 = world.build().with_default::<Transform>().finish();
    let e2 = world.build().with_default::<Transform>().finish();
    let e3 = world.build().with_default::<Transform>().finish();
    let e4 = world.build().with_default::<Transform>().finish();

    {
        let mut arena = world.arena_mut::<Transform>().unwrap();
        Transform::set_parent(&mut arena, e4, Some(e3), false).unwrap();
        Transform::set_parent(&mut arena, e3, Some(e1), false).unwrap();
        Transform::set_parent(&mut arena, e2, Some(e1), false).unwrap();

        assert!(Transform::is_ancestor(&arena, e2, e1));
        assert!(Transform::is_ancestor(&arena, e3, e1));
        assert!(Transform::is_ancestor(&arena, e4, e1));
        assert!(Transform::is_ancestor(&arena, e4, e3));

        assert!(!Transform::is_ancestor(&arena, e1, e1));
        assert!(!Transform::is_ancestor(&arena, e1, e2));
        assert!(!Transform::is_ancestor(&arena, e1, e3));
        assert!(!Transform::is_ancestor(&arena, e1, e4));
        assert!(!Transform::is_ancestor(&arena, e2, e4));
    }

    {
        let t1 = world.fetch::<Transform>(e1).unwrap();
        let t2 = world.fetch::<Transform>(e2).unwrap();
        let t3 = world.fetch::<Transform>(e3).unwrap();
        let t4 = world.fetch::<Transform>(e4).unwrap();

        assert!(t1.is_root());
        assert!(!t2.is_root());
        assert!(!t3.is_root());
        assert!(!t4.is_root());

        assert!(!t1.is_leaf());
        assert!(t2.is_leaf());
        assert!(!t3.is_leaf());
        assert!(t4.is_leaf());
    }
}

#[test]
fn iteration() {
    let mut world = World::new();
    world.register::<Transform>();

    let e1 = world.build().with_default::<Transform>().finish();
    let e2 = world.build().with_default::<Transform>().finish();
    let e3 = world.build().with_default::<Transform>().finish();
    let e4 = world.build().with_default::<Transform>().finish();
    let e5 = world.build().with_default::<Transform>().finish();
    let e6 = world.build().with_default::<Transform>().finish();
    // e1 <- (e2, e3 <- e4 <- (e5, e6))

    let mut arena = world.arena_mut::<Transform>().unwrap();
    Transform::set_parent(&mut arena, e4, Some(e3), false).unwrap();
    Transform::set_parent(&mut arena, e3, Some(e1), false).unwrap();
    Transform::set_parent(&mut arena, e2, Some(e1), false).unwrap();
    Transform::set_parent(&mut arena, e6, Some(e4), false).unwrap();
    Transform::set_parent(&mut arena, e5, Some(e4), false).unwrap();

    assert_eq!(Transform::descendants(&arena, e1).collect::<Vec<_>>(),
               [e2, e3, e4, e5, e6]);
    assert_eq!(Transform::children(&arena, e1).collect::<Vec<_>>(),
               [e2, e3]);
    assert_eq!(Transform::ancestors(&arena, e1).collect::<Vec<_>>(), []);
    assert_eq!(Transform::ancestors(&arena, e2).collect::<Vec<_>>(), [e1]);
    assert_eq!(Transform::ancestors(&arena, e4).collect::<Vec<_>>(),
               [e3, e1]);
    assert_eq!(Transform::ancestors(&arena, e6).collect::<Vec<_>>(),
               [e4, e3, e1]);
}

#[test]
fn random_iteration() {
    let mut generator = XorShiftRng::from_seed([0, 1, 2, 3]);
    let mut world = World::new();
    world.register::<Transform>();

    let mut transforms = vec![];
    for _ in 0..255 {
        transforms.push(world.build().with_default::<Transform>().finish());
    }

    let mut constructed = vec![];
    constructed.push(transforms.pop().unwrap());

    let mut arena = world.arena_mut::<Transform>().unwrap();
    let mut count = 0;
    for i in 0..254 {
        let idx = generator.next_u32() as usize % transforms.len();
        let pidx = generator.next_u32() as usize % constructed.len();

        if pidx == 0 {
            count += 1;
        }

        Transform::set_parent(&mut arena, transforms[idx], Some(constructed[pidx]), false).unwrap();
        let len = Transform::descendants(&arena, constructed[0]).count();
        assert_eq!(len, i + 1);

        constructed.push(transforms[idx]);
        transforms.remove(idx);
    }

    let len = Transform::children(&arena, constructed[0]).count();
    assert_eq!(len, count);

    let len = Transform::descendants(&arena, constructed[0]).count();
    assert_eq!(len, 254);
}