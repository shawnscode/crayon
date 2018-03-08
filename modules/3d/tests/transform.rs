#[macro_use]
extern crate approx;
extern crate crayon;
extern crate crayon_3d;

use crayon::prelude::*;
use crayon::ecs::prelude::*;
use crayon_3d::prelude::*;

pub fn build(world: &mut World) -> Entity {
    world
        .build()
        .with_default::<Node>()
        .with_default::<Transform>()
        .finish()
}

#[test]
pub fn hierachy() {
    let mut world = World::new();
    world.register::<Node>();
    world.register::<Transform>();

    let e1 = build(&mut world);
    let e2 = build(&mut world);
    let e3 = build(&mut world);
    let e4 = build(&mut world);

    let mut tree = world.arena_mut::<Node>();
    let mut arena = world.arena_mut::<Transform>();
    Node::set_parent(&mut tree, e4, Some(e3)).unwrap();
    Node::set_parent(&mut tree, e3, Some(e1)).unwrap();
    Node::set_parent(&mut tree, e2, Some(e1)).unwrap();
    // e1 <- (e2, e3 <- (e4))

    {
        let point = math::Vector3::new(1.0, 0.0, 0.0);
        Transform::set_world_position(&tree, &mut arena, e3, point).unwrap();
        let disp = Transform::world_position(&tree, &arena, e4).unwrap();
        assert_eq!(disp, point);
    }

    {
        let point = [1.0, 0.0, 2.0];
        Transform::set_world_position(&tree, &mut arena, e1, point).unwrap();
        let disp = Transform::world_position(&tree, &arena, e4).unwrap();
        assert_eq!(disp, math::Vector3::new(2.0, 0.0, 2.0));
    }

    {
        assert_eq!(arena.get(e4).unwrap().position(), math::Vector3::zero());
        Node::set_parent(&mut tree, e4, Some(e2)).unwrap();
    }

    {
        let disp = Transform::world_position(&tree, &arena, e4).unwrap();
        assert_eq!(disp, math::Vector3::new(1.0, 0.0, 2.0));
        assert_eq!(arena.get(e4).unwrap().position(), math::Vector3::zero());
    }

    {
        Transform::set_world_scale(&tree, &mut arena, e4, 2.0).unwrap();
        let disp = Transform::world_position(&tree, &arena, e4).unwrap();
        assert_eq!(disp, math::Vector3::new(1.0, 0.0, 2.0));

        let euler = math::Euler::new(math::Deg(0.0), math::Deg(0.0), math::Deg(90.0));
        let rotation = math::Quaternion::from(euler);
        Transform::set_world_rotation(&tree, &mut arena, e4, rotation).unwrap();

        let v = [1.0, 0.0, 0.0];
        let dir = Transform::transform_direction(&tree, &arena, e4, v).unwrap();
        assert!(ulps_eq!(dir, math::Vector3::new(0.0, 1.0, 0.0)));

        let vec = Transform::transform_vector(&tree, &arena, e4, v).unwrap();
        assert!(ulps_eq!(vec, math::Vector3::new(0.0, 2.0, 0.0)));

        let pos = Transform::transform_point(&tree, &arena, e4, v).unwrap();
        assert!(ulps_eq!(pos, math::Vector3::new(1.0, 2.0, 2.0)));
    }
}

#[test]
fn look_at() {
    let mut world = World::new();
    world.register::<Node>();
    world.register::<Transform>();

    let e1 = build(&mut world);

    let tree = world.arena_mut::<Node>();
    let mut arena = world.arena_mut::<Transform>();
    Transform::set_world_position(&tree, &mut arena, e1, [0.0, 0.0, -5.0]).unwrap();

    let v = [0.0, 0.0, 1.0];
    let pos = Transform::transform_point(&tree, &arena, e1, v).unwrap();
    assert!(ulps_eq!(pos, math::Vector3::new(0.0, 0.0, -4.0)));
}
