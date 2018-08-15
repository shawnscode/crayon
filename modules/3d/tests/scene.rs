extern crate crayon;
extern crate crayon_3d;
extern crate rand;

use crayon::ecs::prelude::*;
use crayon_3d::prelude::*;

#[test]
pub fn hierachy() {
    let mut world = World::new();
    let mut scene = SceneGraph::new();

    let e1 = world.create();
    scene.add(e1);

    let e2 = world.create();
    scene.add(e2);

    let e3 = world.create();
    scene.add(e3);

    let e4 = world.create();
    scene.add(e4);

    scene.set_parent(e4, Some(e3)).unwrap();
    scene.set_parent(e3, Some(e1)).unwrap();
    scene.set_parent(e2, Some(e1)).unwrap();

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
}

#[test]
#[should_panic]
pub fn duplicated_add() {
    let mut world = World::new();
    let mut scene = SceneGraph::new();
    let e1 = world.create();
    scene.add(e1);
    scene.add(e1);
}

#[test]
fn iteration() {
    let mut world = World::new();
    let mut scene = SceneGraph::new();

    let e1 = world.create();
    scene.add(e1);
    let e2 = world.create();
    scene.add(e2);
    let e3 = world.create();
    scene.add(e3);
    let e4 = world.create();
    scene.add(e4);
    let e5 = world.create();
    scene.add(e5);
    let e6 = world.create();
    scene.add(e6);

    // e1 <- (e2, e3 <- e4 <- (e5, e6))

    scene.set_parent(e4, Some(e3)).unwrap();
    scene.set_parent(e3, Some(e1)).unwrap();
    scene.set_parent(e2, Some(e1)).unwrap();
    scene.set_parent(e6, Some(e4)).unwrap();
    scene.set_parent(e5, Some(e4)).unwrap();

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
    let mut world = World::new();
    let mut scene = SceneGraph::new();

    let mut nodes = vec![];

    for _ in 0..255 {
        let e = world.create();
        scene.add(e);
        nodes.push(e);
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
            .set_parent(nodes[idx], Some(constructed[pidx]))
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
