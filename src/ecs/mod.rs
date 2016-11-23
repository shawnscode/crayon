//! Entity Component System (ECS)

#[macro_use]
pub mod component;
pub mod world;

pub use self::component::{Component, ComponentStorage, HashMapStorage};
pub use self::world::World;

use super::utils::handle::*;
pub type Entity = Handle;

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{Arc, RwLock};


    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Position {
        x: i32,
        y: i32,
    }

    declare_component!(Position, HashMapStorage);
    declare_component!(Reference, HashMapStorage);

    #[test]
    fn basic() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.create();
        world.assign::<Position>(e1, Position { x: 1, y: 2 });

        {
            let p = world.fetch::<Position>(e1).unwrap();
            assert_eq!(*p, Position { x: 1, y: 2 });
        }

        {
            let p = world.fetch_mut::<Position>(e1).unwrap();
            p.x = 2;
            p.y = 5;
        }

        {
            let p = world.fetch::<Position>(e1).unwrap();
            assert_eq!(*p, Position { x: 2, y: 5 });
        }

        world.remove::<Position>(e1);
        assert_eq!(world.fetch::<Position>(e1), None);
    }

    #[derive(Debug, Clone)]
    struct Reference {
        value: Arc<RwLock<usize>>,
    }

    impl Drop for Reference {
        fn drop(&mut self) {
            *self.value.write().unwrap() += 1;
        }
    }

    #[test]
    fn free() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Reference>();

        let e1 = world.create();
        assert!(world.is_alive(e1));
        assert!(!world.has::<Position>(e1));
        assert_eq!(world.fetch::<Position>(e1), None);

        world.assign::<Position>(e1, Position { x: 1, y: 2 });
        assert!(world.has::<Position>(e1));
        world.fetch::<Position>(e1).unwrap();

        world.free(e1);
        assert!(!world.is_alive(e1));
        assert!(!world.has::<Position>(e1));
        assert_eq!(world.fetch::<Position>(e1), None);

        let mut entities = Vec::new();
        let rc = Arc::new(RwLock::new(0));
        for i in 0..10 {
            let e = world.create();
            let shadow = rc.clone();
            entities.push(e);

            world.assign::<Reference>(e, Reference { value: shadow });
            if i % 2 == 0 {
                world.assign::<Position>(e, Position { x: 1, y: 2 });
            }
        }

        assert_eq!(*rc.read().unwrap(), 0);
        for i in 0..10 {
            world.free(entities[i]);
            assert_eq!(*rc.read().unwrap(), i + 1);
        }
        assert_eq!(*rc.read().unwrap(), 10);
    }

    #[test]
    fn duplicated_assign() {
        let mut world = World::new();
        world.register::<Position>();

        let e1 = world.create();
        assert!(world.assign::<Position>(e1, Position { x: 1, y: 2 }) == None);
        assert!(world.assign::<Position>(e1, Position { x: 2, y: 4 }) ==
                Some(Position { x: 1, y: 2 }));

        assert!(world.fetch::<Position>(e1) == Some(&Position { x: 2, y: 4 }))
    }
}