extern crate crayon;

use crayon::utils::prelude::*;

#[test]
fn basic() {
    let mut set = ObjectPool::<Handle, i32>::new();

    let e1 = set.create(3);
    assert_eq!(set.get(e1), Some(&3));
    assert_eq!(set.len(), 1);
    assert_eq!(set.free(e1), Some(3));
    assert_eq!(set.len(), 0);
    assert_eq!(set.get(e1), None);
    assert_eq!(set.free(e1), None);
    assert_eq!(set.len(), 0);
}

#[test]
fn iterator() {
    let mut set = ObjectPool::<Handle, i32>::new();
    for i in 0..10 {
        set.create(i);
    }

    assert!(set.iter().count() == 10);

    for (i, v) in set.keys().enumerate() {
        assert_eq!(v, Handle::new(i as u32, 1));
    }

    for (i, &v) in set.values().enumerate() {
        assert_eq!(v, i as i32);
    }

    for v in set.values_mut() {
        *v += 1;
    }

    for (i, &v) in set.values().enumerate() {
        assert_eq!(v, (i + 1) as i32);
    }
}
