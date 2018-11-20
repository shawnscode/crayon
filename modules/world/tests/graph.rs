extern crate crayon;
extern crate crayon_world;
extern crate rand;

use crayon::prelude::*;
use crayon::*;
use crayon_world::prelude::*;
use crayon_world::renderable::headless::HeadlessRenderer;

#[test]
pub fn hierachy() {
    let mut scene = Scene::new(HeadlessRenderer::new());
    let e1 = scene.create("e1");
    let e2 = scene.create("e2");
    let e3 = scene.create("e3");
    let e4 = scene.create("e4");

    scene.set_parent(e4, e3, false).unwrap();
    scene.set_parent(e3, e1, false).unwrap();
    scene.set_parent(e2, e1, false).unwrap();
    // e1 <- (e2, e3 <- (e4))

    assert!(scene.is_ancestor(e2, e1));
    assert!(scene.is_ancestor(e3, e1));
    assert!(scene.is_ancestor(e4, e1));
    assert!(scene.is_ancestor(e4, e3));

    assert!(!scene.is_ancestor(e1, e1));
    assert!(!scene.is_ancestor(e1, e2));
    assert!(!scene.is_ancestor(e1, e3));
    assert!(!scene.is_ancestor(e1, e4));
    assert!(!scene.is_ancestor(e2, e4));

    assert!(scene.is_root(e1));
    assert!(!scene.is_root(e2));
    assert!(!scene.is_root(e3));
    assert!(!scene.is_root(e4));

    assert!(!scene.is_leaf(e1));
    assert!(scene.is_leaf(e2));
    assert!(!scene.is_leaf(e3));
    assert!(scene.is_leaf(e4));

    let point = [1.0, 0.0, 0.0];
    scene.set_position(e3, point);
    assert_ulps_eq!(scene.position(e4).unwrap(), point.into());

    let point = [1.0, 0.0, 2.0];
    scene.set_position(e1, point);
    assert_ulps_eq!(scene.position(e4).unwrap(), [2.0, 0.0, 2.0].into());

    assert_ulps_eq!(scene.local_position(e4).unwrap(), [0.0, 0.0, 0.0].into());
    scene.set_parent(e4, Some(e2), false).unwrap();
    assert_ulps_eq!(scene.position(e4).unwrap(), [1.0, 0.0, 2.0].into());
    assert_ulps_eq!(scene.local_position(e4).unwrap(), [0.0, 0.0, 0.0].into());

    scene.set_local_position(e2, [1.0, 0.0, 0.0]);
    let euler = Euler::new(Deg(0.0), Deg(90.0), Deg(0.0));
    scene.set_rotation(e1, euler);
    assert_ulps_eq!(scene.position(e2).unwrap(), [1.0, 0.0, 1.0].into());
}

#[test]
fn remove() {
    let mut scene = Scene::new(HeadlessRenderer::new());
    let e1 = scene.create("e1");
    let e2 = scene.create("e2");
    let e3 = scene.create("e3");
    let e4 = scene.create("e4");
    let e5 = scene.create("e5");
    let e6 = scene.create("e6");

    scene.set_parent(e2, e1, false).unwrap();
    scene.set_parent(e3, e1, false).unwrap();
    scene.set_parent(e4, e3, false).unwrap();
    scene.set_parent(e5, e3, false).unwrap();
    scene.set_parent(e6, e5, false).unwrap();
    // e1 <- (e2, e3 <- (e4, e5 <- e6))

    assert!(scene.len() == 6);

    scene.delete(e3);
    assert!(scene.contains(e1));
    assert!(scene.contains(e2));
    assert!(!scene.contains(e3));
    assert!(!scene.contains(e4));
    assert!(!scene.contains(e5));
    assert!(!scene.contains(e6));
    assert!(scene.len() == 2);
}

#[test]
fn transform() {
    let mut scene = Scene::new(HeadlessRenderer::new());
    let e1 = scene.create("e1");

    scene.set_scale(e1, 2.0);
    scene.set_position(e1, [1.0, 0.0, 2.0]);

    let euler = Euler::new(Deg(0.0), Deg(0.0), Deg(90.0));
    let rotation = Quaternion::from(euler);
    scene.set_rotation(e1, rotation);

    let v = [1.0, 0.0, 0.0];
    let transform = scene.transform(e1).unwrap();
    assert_ulps_eq!(transform.transform_direction(v), [0.0, 1.0, 0.0].into());
    assert_ulps_eq!(transform.transform_vector(v), [0.0, 2.0, 0.0].into());
    assert_ulps_eq!(transform.transform_point(v), [1.0, 2.0, 2.0].into());
}

#[test]
fn keep_world_pose() {
    // Hierachy changes might have affects on node's transform.
    let mut scene = Scene::new(HeadlessRenderer::new());
    let e1 = scene.create("e1");
    let e2 = scene.create("e2");
    let e3 = scene.create("e3");

    scene.set_position(e1, [0.0, 1.0, 0.0]);
    assert_ulps_eq!(scene.position(e1).unwrap(), [0.0, 1.0, 0.0].into());
    assert_ulps_eq!(scene.local_position(e1).unwrap(), [0.0, 1.0, 0.0].into());

    scene.set_position(e2, [1.0, 0.0, 0.0]);
    scene.set_position(e3, [0.0, 0.0, 1.0]);

    scene.set_parent(e2, e1, false).unwrap();
    assert_ulps_eq!(scene.local_position(e2).unwrap(), [1.0, 0.0, 0.0].into());
    assert_ulps_eq!(scene.position(e2).unwrap(), [1.0, 1.0, 0.0].into());

    scene.remove_from_parent(e2, true).unwrap();
    assert_ulps_eq!(scene.position(e2).unwrap(), [1.0, 1.0, 0.0].into());

    scene.set_parent(e3, e1, true).unwrap();
    assert_ulps_eq!(scene.local_position(e3).unwrap(), [0.0, -1.0, 1.0].into());
    assert_ulps_eq!(scene.position(e3).unwrap(), [0.0, 0.0, 1.0].into());

    scene.remove_from_parent(e3, false).unwrap();
    assert_ulps_eq!(scene.position(e3).unwrap(), [0.0, -1.0, 1.0].into());
}

#[test]
fn look_at() {
    let mut scene = Scene::new(HeadlessRenderer::new());
    let e1 = scene.create("e1");
    let euler = Euler::new(Deg(0.0), Deg(0.0), Deg(0.0));
    assert_ulps_eq!(scene.rotation(e1).unwrap(), euler.into());

    scene.set_position(e1, [0.0, 0.0, -5.0]);
    scene.look_at(e1, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let euler = Euler::new(Deg(0.0), Deg(0.0), Deg(0.0));
    assert_ulps_eq!(scene.rotation(e1).unwrap(), euler.into());

    scene.set_position(e1, [0.0, 0.0, 5.0]);
    scene.look_at(e1, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let euler = Euler::new(Deg(0.0), Deg(180.0), Deg(0.0));
    assert_ulps_eq!(scene.rotation(e1).unwrap(), euler.into());

    scene.set_position(e1, [1.0, 0.0, 1.0]);
    scene.look_at(e1, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let euler = Euler::new(Deg(0.0), Deg(225.0), Deg(0.0));
    assert_ulps_eq!(scene.rotation(e1).unwrap(), euler.into());
}

#[test]
fn iteration() {
    let mut scene = Scene::new(HeadlessRenderer::new());
    let e1 = scene.create("e1");
    let e2 = scene.create("e2");
    let e3 = scene.create("e3");
    let e4 = scene.create("e4");
    let e5 = scene.create("e5");
    let e6 = scene.create("e6");

    // e1 <- (e2, e3 <- e4 <- (e5, e6))

    scene.set_parent(e4, e3, false).unwrap();
    scene.set_parent(e3, e1, false).unwrap();
    scene.set_parent(e2, e1, false).unwrap();
    scene.set_parent(e6, e4, false).unwrap();
    scene.set_parent(e5, e4, false).unwrap();

    assert_eq!(
        scene.descendants(e1).collect::<Vec<_>>(),
        [e2, e3, e4, e5, e6]
    );

    assert_eq!(scene.children(e1).collect::<Vec<_>>(), [e2, e3]);
    assert_eq!(scene.ancestors(e1).collect::<Vec<_>>(), []);
    assert_eq!(scene.ancestors(e2).collect::<Vec<_>>(), [e1]);
    assert_eq!(scene.ancestors(e4).collect::<Vec<_>>(), [e3, e1]);
    assert_eq!(scene.ancestors(e6).collect::<Vec<_>>(), [e4, e3, e1]);
}

#[test]
fn random_iteration() {
    let mut scene = Scene::new(HeadlessRenderer::new());

    let mut nodes = vec![];
    for _ in 0..255 {
        nodes.push(scene.create(""));
    }

    let mut constructed = vec![];
    constructed.push(nodes.pop().unwrap());

    let mut count = 0;
    for i in 0..254 {
        let idx = rand::random::<usize>() % nodes.len();
        let pidx = rand::random::<usize>() % constructed.len();

        if pidx == 0 {
            count += 1;
        }

        scene
            .set_parent(nodes[idx], constructed[pidx], false)
            .unwrap();

        let len = scene.descendants(constructed[0]).count();
        assert_eq!(len, i + 1);

        constructed.push(nodes[idx]);
        nodes.remove(idx);
    }

    let len = scene.children(constructed[0]).count();
    assert_eq!(len, count);

    let len = scene.descendants(constructed[0]).count();
    assert_eq!(len, 254);
}
