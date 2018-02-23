extern crate crayon;
extern crate crayon_scene;
extern crate rand;

use crayon::prelude::*;
use crayon_scene::prelude::*;

use rand::{Rng, SeedableRng, XorShiftRng};

#[test]
pub fn hierachy() {
    let mut world = World::new();
    world.register::<Node>();

    let e1 = world.build().with_default::<Node>().finish();
    let e2 = world.build().with_default::<Node>().finish();
    let e3 = world.build().with_default::<Node>().finish();
    let e4 = world.build().with_default::<Node>().finish();

    {
        let mut arena = world.arena_mut::<Node>();
        Node::set_parent(&mut arena, e4, Some(e3)).unwrap();
        Node::set_parent(&mut arena, e3, Some(e1)).unwrap();
        Node::set_parent(&mut arena, e2, Some(e1)).unwrap();

        assert!(Node::is_ancestor(&arena, e2, e1));
        assert!(Node::is_ancestor(&arena, e3, e1));
        assert!(Node::is_ancestor(&arena, e4, e1));
        assert!(Node::is_ancestor(&arena, e4, e3));

        assert!(!Node::is_ancestor(&arena, e1, e1));
        assert!(!Node::is_ancestor(&arena, e1, e2));
        assert!(!Node::is_ancestor(&arena, e1, e3));
        assert!(!Node::is_ancestor(&arena, e1, e4));
        assert!(!Node::is_ancestor(&arena, e2, e4));
    }

    {
        let t1 = world.get::<Node>(e1).unwrap();
        let t2 = world.get::<Node>(e2).unwrap();
        let t3 = world.get::<Node>(e3).unwrap();
        let t4 = world.get::<Node>(e4).unwrap();

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
    world.register::<Node>();

    let e1 = world.build().with_default::<Node>().finish();
    let e2 = world.build().with_default::<Node>().finish();
    let e3 = world.build().with_default::<Node>().finish();
    let e4 = world.build().with_default::<Node>().finish();
    let e5 = world.build().with_default::<Node>().finish();
    let e6 = world.build().with_default::<Node>().finish();
    // e1 <- (e2, e3 <- e4 <- (e5, e6))

    let mut arena = world.arena_mut::<Node>();
    Node::set_parent(&mut arena, e4, Some(e3)).unwrap();
    Node::set_parent(&mut arena, e3, Some(e1)).unwrap();
    Node::set_parent(&mut arena, e2, Some(e1)).unwrap();
    Node::set_parent(&mut arena, e6, Some(e4)).unwrap();
    Node::set_parent(&mut arena, e5, Some(e4)).unwrap();

    assert_eq!(
        Node::descendants(&arena, e1).collect::<Vec<_>>(),
        [e2, e3, e4, e5, e6]
    );
    assert_eq!(Node::children(&arena, e1).collect::<Vec<_>>(), [e2, e3]);
    assert_eq!(Node::ancestors(&arena, e1).collect::<Vec<_>>(), []);
    assert_eq!(Node::ancestors(&arena, e2).collect::<Vec<_>>(), [e1]);
    assert_eq!(Node::ancestors(&arena, e4).collect::<Vec<_>>(), [e3, e1]);
    assert_eq!(
        Node::ancestors(&arena, e6).collect::<Vec<_>>(),
        [e4, e3, e1]
    );
}

#[test]
fn random_iteration() {
    let mut generator = XorShiftRng::from_seed([0, 1, 2, 3]);
    let mut world = World::new();
    world.register::<Node>();

    let mut transforms = vec![];
    for _ in 0..255 {
        transforms.push(world.build().with_default::<Node>().finish());
    }

    let mut constructed = vec![];
    constructed.push(transforms.pop().unwrap());

    let mut arena = world.arena_mut::<Node>();
    let mut count = 0;
    for i in 0..254 {
        let idx = generator.next_u32() as usize % transforms.len();
        let pidx = generator.next_u32() as usize % constructed.len();

        if pidx == 0 {
            count += 1;
        }

        Node::set_parent(&mut arena, transforms[idx], Some(constructed[pidx])).unwrap();
        let len = Node::descendants(&arena, constructed[0]).count();
        assert_eq!(len, i + 1);

        constructed.push(transforms[idx]);
        transforms.remove(idx);
    }

    let len = Node::children(&arena, constructed[0]).count();
    assert_eq!(len, count);

    let len = Node::descendants(&arena, constructed[0]).count();
    assert_eq!(len, 254);
}
