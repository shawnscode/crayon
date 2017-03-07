#[macro_use]
extern crate crayon;
#[macro_use]
extern crate lazy_static;

use crayon::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Position {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Velocity {
    x: u32,
    y: u32,
}

declare_component!(Position, VecStorage);
declare_component!(Velocity, VecStorage);

#[test]
fn iterate() {
    // let master = ThreadPool::new(4);
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();

    for i in 0..50 {
        let ent = world.build().with_default::<Position>().finish();
        let velocity = Velocity {
            x: i,
            y: ent.version(),
        };
        world.assign::<Velocity>(ent, velocity);

        if i % 3 == 0 {
            world.free(ent);
        }
    }

    // world.view_with_r1w1::<Velocity, Position>().for_each(&master,
    //                                                       100,
    //                                                       &|item| {
    //                                                           item.writables.x += item.readables.x;
    //                                                           item.writables.y += item.readables.y;
    //                                                       });
}