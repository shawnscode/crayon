#[macro_use]
extern crate crayon;
extern crate crayon_3d;

use crayon::prelude::*;
use crayon_3d::prelude::*;

#[test]
fn transform() {
    let mut e1 = Transform::default();
    let euler = math::Euler::new(math::Deg(0.0), math::Deg(0.0), math::Deg(90.0));
    e1.scale = 2.0;
    e1.position = [1.0, 0.0, 2.0].into();
    e1.rotation = euler.into();

    let v = [1.0, 0.0, 0.0];
    assert_ulps_eq!(e1.transform_direction(v), [0.0, 1.0, 0.0].into());
    assert_ulps_eq!(e1.transform_vector(v), [0.0, 2.0, 0.0].into());
    assert_ulps_eq!(e1.transform_point(v), [1.0, 2.0, 2.0].into());
}

#[test]
fn concat() {
    let mut e1 = Transform::default();
    e1.position = [0.0, 0.0, 1.0].into();

    let mut e2 = Transform::default();
    let euler = math::Euler::new(math::Deg(0.0), math::Deg(90.0), math::Deg(0.0));
    e2.rotation = euler.into();

    let e3 = e2 * e1;
    assert_ulps_eq!(e3.position, [1.0, 0.0, 0.0].into());
}

#[test]
fn inverse() {
    let mut e1 = Transform::default();
    e1.position = [0.0, 0.0, 1.0].into();
    let euler = math::Euler::new(math::Deg(0.0), math::Deg(90.0), math::Deg(0.0));
    e1.rotation = euler.into();

    let v = e1.inverse().unwrap() * e1;
    assert_ulps_eq!(v.position, [0.0, 0.0, 0.0].into());
    assert_ulps_eq!(v.scale, 1.0);
    assert_ulps_eq!(v.rotation, math::Quaternion::one());
}
