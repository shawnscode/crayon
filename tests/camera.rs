extern crate crayon;
#[macro_use]
extern crate approx;

use crayon::prelude::*;

#[test]
fn camera() {
    let mut world = World::new();
    world.register::<Transform>();
    world.register::<Camera>();

    let camera = world
        .build()
        .with_default::<Transform>()
        .with_default::<Camera>()
        .finish();

    {
        let mut arena = world.arena_mut::<Transform>().unwrap();
        Transform::set_world_position(&mut arena, camera, math::Vector3::new(0.0, 0.0, -5.0))
            .unwrap();
    }

    {
        let view_matrix = Camera::view_matrix(&world.arena_mut::<Transform>().unwrap(), camera)
            .unwrap();

        let point = math::Point3::new(0.0, 0.0, 1.0);
        let view_point = view_matrix.transform_point(point);

        assert!(ulps_eq!(view_point, math::Point3::new(0.0, 0.0, 6.0)));
    }

    {
        let mut arena = world.arena_mut::<Transform>().unwrap();
        Transform::set_world_rotation(&mut arena,
                                      camera,
                                      math::Quaternion::from(math::Euler {
                                                                 x: math::Deg(0f32),
                                                                 y: math::Deg(0f32),
                                                                 z: math::Deg(90f32),
                                                             }))
                .unwrap();
    }

    {
        let view_matrix = Camera::view_matrix(&world.arena_mut::<Transform>().unwrap(), camera)
            .unwrap();
        let point = math::Point3::new(1.0, 0.0, 0.0);
        let view_point = view_matrix.transform_point(point);

        assert!(ulps_eq!(view_point, math::Point3::new(0.0, -1.0, 5.0)));
    }
}