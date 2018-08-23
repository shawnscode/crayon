extern crate crayon;
extern crate crayon_3d;

use crayon::utils::handle_pool::HandlePool;

use crayon_3d::scene::SceneGraph;
use crayon_3d::tags::{self, Tags};
use crayon_3d::Entity;

struct Testbed {
    world: HandlePool,
    scene: SceneGraph,
    tags: Tags,
}

impl Testbed {
    fn new() -> Testbed {
        Testbed {
            world: HandlePool::new(),
            scene: SceneGraph::new(),
            tags: Tags::new(),
        }
    }

    fn create(&mut self, name: &str) -> Entity {
        let ent = self.world.create().into();
        self.scene.add(ent);
        self.tags.add(ent, name);
        ent
    }

    fn find_by_name(&self, name: &str) -> Option<Entity> {
        tags::find_by_name(&self.scene, &self.tags, name)
    }
}

#[test]
fn find_by_name() {
    let mut testbed = Testbed::new();

    let e1 = testbed.create("room.obj");
    let e2 = testbed.create("floor");
    let e3 = testbed.create("tallBox");
    let e4 = testbed.create("shortBox");

    testbed.scene.set_parent(e2, e1, false).unwrap();
    testbed.scene.set_parent(e3, e1, false).unwrap();
    testbed.scene.set_parent(e4, e3, false).unwrap();

    assert_eq!(Some(e1), testbed.find_by_name("room.obj"));
    assert_eq!(Some(e1), testbed.find_by_name("room.obj/"));
    assert_eq!(Some(e1), testbed.find_by_name("room.obj//"));
    assert_eq!(Some(e1), testbed.find_by_name("/room.obj"));
    assert_eq!(Some(e1), testbed.find_by_name("//room.obj"));
    assert_eq!(Some(e1), testbed.find_by_name("/room.obj//"));

    assert_eq!(Some(e2), testbed.find_by_name("room.obj/floor"));
    assert_eq!(Some(e3), testbed.find_by_name("room.obj/tallBox"));
    assert_eq!(Some(e4), testbed.find_by_name("room.obj/tallBox/shortBox"));
}
