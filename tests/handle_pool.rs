extern crate crayon;
extern crate rand;

use std::cmp::min;

use crayon::utils::prelude::*;

#[test]
fn handle_set() {
    let mut set: HandlePool<Handle> = HandlePool::new();
    assert_eq!(set.len(), 0);

    // Spawn entities.
    let e1 = set.create();
    assert!(e1.is_valid());
    assert!(set.contains(e1));
    assert_eq!(set.len(), 1);

    let mut e2 = e1;
    assert!(set.contains(e2));
    assert_eq!(set.len(), 1);

    // Invalidate entities.
    e2.invalidate();
    assert!(!e2.is_valid());
    assert!(!set.contains(e2));
    assert!(set.contains(e1));

    // Free entities.
    let e2 = e1;
    set.free(e2);
    assert!(!set.contains(e2));
    assert!(!set.contains(e1));
    assert_eq!(set.len(), 0);
}

#[test]
fn retain() {
    let mut set: HandlePool<Handle> = HandlePool::new();
    for _ in 0..10 {
        set.create();
    }

    set.retain(|e| e.index() % 2 == 0);

    for v in &set {
        assert!(v.index() % 2 == 0);
    }
}

#[test]
fn index_reuse() {
    let mut set: HandlePool<Handle> = HandlePool::new();

    assert_eq!(set.len(), 0);

    let mut v = vec![];
    for _ in 0..10 {
        v.push(set.create());
    }

    assert_eq!(set.len(), 10);
    for e in v.iter() {
        set.free(*e);
    }

    for _ in 0..10 {
        let e = set.create();
        assert!((*e as usize) < v.len());
        assert!(v[*e as usize].version() != e.version());
    }
}

#[test]
fn index_compact_reuse() {
    let mut set: HandlePool<Handle> = HandlePool::new();

    let mut v = vec![];
    for _ in 0..5 {
        for _ in 0..50 {
            v.push(set.create());
        }

        let size = v.len() / 2;
        for _ in 0..size {
            let len = v.len();
            set.free(v.swap_remove(rand::random::<usize>() % len));
        }
    }

    for i in v {
        set.free(i);
    }

    for index in 0..50 {
        let handle = set.create();
        assert_eq!(handle.index(), index);
    }
}

#[test]
fn iter() {
    let mut set: HandlePool<Handle> = HandlePool::new();
    let mut v = vec![];

    for m in 2..3 {
        for _ in 0..10 {
            v.push(set.create())
        }

        for i in 0..10 {
            if i % m == 0 {
                let index = i % v.len();
                set.free(v[index]);
                v.remove(index);
            }
        }
    }

    v.sort_by(|lhs, rhs| lhs.index().cmp(&rhs.index()));
    let mut iter = set.iter();
    let test_split_at = |stride| {
        let iter = set.iter();
        let (mut s1, mut s2) = iter.split_at(stride);
        assert_eq!(s1.len(), min(stride, iter.len()));
        assert_eq!(s2.len(), iter.len() - min(stride, iter.len()));

        for handle in &v {
            if let Some(v) = s1.next() {
                assert_eq!(*handle, v);
            } else {
                assert_eq!(*handle, s2.next().unwrap());
            }
        }
    };

    test_split_at(0);
    test_split_at(1);
    test_split_at(iter.len() - 1);
    test_split_at(iter.len());
    test_split_at(iter.len() + 1);
    test_split_at(iter.len() * 2);

    for handle in &v {
        assert_eq!(*handle, iter.next().unwrap());
    }
}
