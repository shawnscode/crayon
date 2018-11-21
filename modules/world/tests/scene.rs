extern crate crayon;
extern crate crayon_world;

use crayon_world::prelude::*;
use crayon_world::renderable::headless::HeadlessRenderer;

#[test]
fn find() {
    let mut scene = Scene::new(HeadlessRenderer::new());

    let e1 = scene.create("room.obj");
    let e2 = scene.create("floor");
    let e3 = scene.create("tallBox");
    let e4 = scene.create("shortBox");

    scene.set_parent(e2, e1, false).unwrap();
    scene.set_parent(e3, e1, false).unwrap();
    scene.set_parent(e4, e3, false).unwrap();

    assert_eq!(scene.find("room.obj"), Some(e1));
    assert_eq!(scene.find("room.obj/"), Some(e1));
    assert_eq!(scene.find("room.obj//"), Some(e1));
    assert_eq!(scene.find("/room.obj"), Some(e1));
    assert_eq!(scene.find("//room.obj"), Some(e1));
    assert_eq!(scene.find("/room.obj//"), Some(e1));

    assert_eq!(scene.find("room.obj/floor"), Some(e2));
    assert_eq!(scene.find("room.obj/tallBox"), Some(e3));
    assert_eq!(scene.find("room.obj/tallBox/shortBox"), Some(e4));

    assert_eq!(scene.find("room.obj/blahblah"), None);
}

#[test]
fn instantiate() {
    use crayon_world::assets::prefab::PrefabNode;
    crayon::application::oneshot().unwrap();
    crayon_world::setup().unwrap();

    let mut prefab = Prefab {
        nodes: Vec::new(),
        universe_meshes: Vec::new(),
        meshes: Vec::new(),
    };

    prefab.nodes.push(PrefabNode {
        name: "room.obj".into(),
        local_transform: Transform::default(),
        first_child: Some(1),
        next_sib: None,
        mesh_renderer: None,
    });

    prefab.nodes.push(PrefabNode {
        name: "floor".into(),
        local_transform: Transform::default(),
        first_child: Some(2),
        next_sib: None,
        mesh_renderer: None,
    });

    prefab.nodes.push(PrefabNode {
        name: "tallBox".into(),
        local_transform: Transform::default(),
        first_child: None,
        next_sib: Some(3),
        mesh_renderer: None,
    });

    prefab.nodes.push(PrefabNode {
        name: "shortBox".into(),
        local_transform: Transform::default(),
        first_child: None,
        next_sib: None,
        mesh_renderer: None,
    });

    let template = crayon_world::create_prefab(prefab).unwrap();

    let mut scene = Scene::new(HeadlessRenderer::new());
    let e1 = scene.instantiate(template).unwrap();

    assert_eq!(scene.find("room.obj"), Some(e1));
    assert!(scene.find("room.obj/floor/tallBox").is_some());

    assert_eq!(
        scene.find_from(e1, "floor/tallBox"),
        scene.find("room.obj/floor/tallBox")
    );
}
