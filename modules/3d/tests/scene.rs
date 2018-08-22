extern crate crayon;
extern crate crayon_3d;
extern crate rand;

use crayon::math::*;
use crayon::utils::handle_pool::HandlePool;
use crayon_3d::prelude::*;

struct Testbed {
    world: HandlePool,
    scene: SceneGraph,
}

impl Testbed {
    fn new() -> Testbed {
        Testbed {
            world: HandlePool::new(),
            scene: SceneGraph::new(),
        }
    }

    fn create(&mut self) -> Entity {
        let ent = self.world.create().into();
        self.scene.add(ent);
        ent
    }
}

impl ::std::ops::Deref for Testbed {
    type Target = SceneGraph;

    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}

impl ::std::ops::DerefMut for Testbed {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene
    }
}

#[test]
pub fn hierachy() {
    let mut testbed = Testbed::new();
    let e1 = testbed.create();
    let e2 = testbed.create();
    let e3 = testbed.create();
    let e4 = testbed.create();

    testbed.set_parent(e4, e3, false).unwrap();
    testbed.set_parent(e3, e1, false).unwrap();
    testbed.set_parent(e2, e1, false).unwrap();
    // e1 <- (e2, e3 <- (e4))

    assert!(testbed.is_ancestor(e2, e1));
    assert!(testbed.is_ancestor(e3, e1));
    assert!(testbed.is_ancestor(e4, e1));
    assert!(testbed.is_ancestor(e4, e3));

    assert!(!testbed.is_ancestor(e1, e1));
    assert!(!testbed.is_ancestor(e1, e2));
    assert!(!testbed.is_ancestor(e1, e3));
    assert!(!testbed.is_ancestor(e1, e4));
    assert!(!testbed.is_ancestor(e2, e4));

    assert!(testbed.is_root(e1));
    assert!(!testbed.is_root(e2));
    assert!(!testbed.is_root(e3));
    assert!(!testbed.is_root(e4));

    assert!(!testbed.is_leaf(e1));
    assert!(testbed.is_leaf(e2));
    assert!(!testbed.is_leaf(e3));
    assert!(testbed.is_leaf(e4));

    {
        let point = [1.0, 0.0, 0.0];
        testbed.set_position(e3, point);
        assert_ulps_eq!(testbed.position(e4).unwrap(), point.into());
    }

    {
        let point = [1.0, 0.0, 2.0];
        testbed.set_position(e1, point);
        assert_ulps_eq!(testbed.position(e4).unwrap(), [2.0, 0.0, 2.0].into());
    }

    {
        assert_ulps_eq!(testbed.local_position(e4).unwrap(), [0.0, 0.0, 0.0].into());
        testbed.set_parent(e4, Some(e2), false).unwrap();
        assert_ulps_eq!(testbed.position(e4).unwrap(), [1.0, 0.0, 2.0].into());
        assert_ulps_eq!(testbed.local_position(e4).unwrap(), [0.0, 0.0, 0.0].into());
    }
}

#[test]
fn transform() {
    let mut testbed = Testbed::new();
    let e1 = testbed.create();

    testbed.set_scale(e1, 2.0);
    testbed.set_position(e1, [1.0, 0.0, 2.0]);

    let euler = Euler::new(Deg(0.0), Deg(0.0), Deg(90.0));
    let rotation = Quaternion::from(euler);
    testbed.set_rotation(e1, rotation);

    let v = [1.0, 0.0, 0.0];
    let transform = testbed.transform(e1).unwrap();
    assert_ulps_eq!(transform.transform_direction(v), [0.0, 1.0, 0.0].into());
    assert_ulps_eq!(transform.transform_vector(v), [0.0, 2.0, 0.0].into());
    assert_ulps_eq!(transform.transform_point(v), [1.0, 2.0, 2.0].into());
}

#[test]
fn keep_world_pose() {
    // Hierachy changes might have affects on node's transform.
    let mut testbed = Testbed::new();
    let e1 = testbed.create();
    let e2 = testbed.create();
    let e3 = testbed.create();

    testbed.set_position(e1, [0.0, 1.0, 0.0]);
    assert_ulps_eq!(testbed.position(e1).unwrap(), [0.0, 1.0, 0.0].into());
    assert_ulps_eq!(testbed.local_position(e1).unwrap(), [0.0, 1.0, 0.0].into());

    testbed.set_position(e2, [1.0, 0.0, 0.0]);
    testbed.set_position(e3, [0.0, 0.0, 1.0]);

    testbed.set_parent(e2, e1, false).unwrap();
    assert_ulps_eq!(testbed.local_position(e2).unwrap(), [1.0, 0.0, 0.0].into());
    assert_ulps_eq!(testbed.position(e2).unwrap(), [1.0, 1.0, 0.0].into());

    testbed.remove_from_parent(e2, true).unwrap();
    assert_ulps_eq!(testbed.position(e2).unwrap(), [1.0, 1.0, 0.0].into());

    testbed.set_parent(e3, e1, true).unwrap();
    assert_ulps_eq!(testbed.local_position(e3).unwrap(), [0.0, -1.0, 1.0].into());
    assert_ulps_eq!(testbed.position(e3).unwrap(), [0.0, 0.0, 1.0].into());

    testbed.remove_from_parent(e3, false).unwrap();
    assert_ulps_eq!(testbed.position(e3).unwrap(), [0.0, -1.0, 1.0].into());
}

#[test]
fn look_at() {
    let mut testbed = Testbed::new();
    let e1 = testbed.create();
    let euler = Euler::new(Deg(0.0), Deg(0.0), Deg(0.0));
    assert_ulps_eq!(testbed.rotation(e1).unwrap(), euler.into());

    testbed.set_position(e1, [0.0, 0.0, -5.0]);
    testbed.look_at(e1, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let euler = Euler::new(Deg(0.0), Deg(0.0), Deg(0.0));
    assert_ulps_eq!(testbed.rotation(e1).unwrap(), euler.into());

    testbed.set_position(e1, [0.0, 0.0, 5.0]);
    testbed.look_at(e1, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let euler = Euler::new(Deg(0.0), Deg(180.0), Deg(0.0));
    assert_ulps_eq!(testbed.rotation(e1).unwrap(), euler.into());

    testbed.set_position(e1, [1.0, 0.0, 1.0]);
    testbed.look_at(e1, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let euler = Euler::new(Deg(0.0), Deg(225.0), Deg(0.0));
    assert_ulps_eq!(testbed.rotation(e1).unwrap(), euler.into());
}

#[test]
#[should_panic]
pub fn duplicated_add() {
    let mut world = HandlePool::new();
    let mut scene = SceneGraph::new();

    let e1 = world.create().into();
    scene.add(e1);
    scene.add(e1);
}

#[test]
fn iteration() {
    let mut testbed = Testbed::new();
    let e1 = testbed.create();
    let e2 = testbed.create();
    let e3 = testbed.create();
    let e4 = testbed.create();
    let e5 = testbed.create();
    let e6 = testbed.create();

    // e1 <- (e2, e3 <- e4 <- (e5, e6))

    testbed.set_parent(e4, e3, false).unwrap();
    testbed.set_parent(e3, e1, false).unwrap();
    testbed.set_parent(e2, e1, false).unwrap();
    testbed.set_parent(e6, e4, false).unwrap();
    testbed.set_parent(e5, e4, false).unwrap();

    assert_eq!(
        testbed.descendants(e1).collect::<Vec<_>>(),
        [e2, e3, e4, e5, e6]
    );

    assert_eq!(testbed.children(e1).collect::<Vec<_>>(), [e2, e3]);
    assert_eq!(testbed.ancestors(e1).collect::<Vec<_>>(), []);
    assert_eq!(testbed.ancestors(e2).collect::<Vec<_>>(), [e1]);
    assert_eq!(testbed.ancestors(e4).collect::<Vec<_>>(), [e3, e1]);
    assert_eq!(testbed.ancestors(e6).collect::<Vec<_>>(), [e4, e3, e1]);
}

#[test]
fn random_iteration() {
    let mut testbed = Testbed::new();

    let mut nodes = vec![];
    for _ in 0..255 {
        nodes.push(testbed.create());
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

        testbed
            .set_parent(nodes[idx], constructed[pidx], false)
            .unwrap();

        let len = testbed.descendants(constructed[0]).count();
        assert_eq!(len, i + 1);

        constructed.push(nodes[idx]);
        nodes.remove(idx);
    }

    let len = testbed.children(constructed[0]).count();
    assert_eq!(len, count);

    let len = testbed.descendants(constructed[0]).count();
    assert_eq!(len, 254);
}
