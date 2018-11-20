// extern crate crayon;
// extern crate crayon_3d;

// use crayon::utils::handle_pool::HandlePool;

// use crayon_3d::prelude::*;
// use crayon_3d::renderers::Renderable;
// use crayon_3d::tags::Tags;
// use crayon_3d::world_impl;

// struct Testbed {
//     entities: HandlePool<Entity>,
//     scene: SceneGraph,
//     tags: Tags,
//     renderables: Renderable,
// }

// impl Testbed {
//     fn new() -> Testbed {
//         Testbed {
//             entities: HandlePool::new(),
//             scene: SceneGraph::new(),
//             tags: Tags::new(),
//             renderables: Renderable::new(),
//         }
//     }

//     fn create(&mut self, name: &str) -> Entity {
//         let e1 = world_impl::create(&mut self.entities, &mut self.scene);
//         self.tags.add(e1, name);
//         e1
//     }

//     fn find(&self, name: &str) -> Option<Entity> {
//         world_impl::find(&self.scene, &self.tags, name)
//     }

//     fn instantiate(&mut self, prefab: &Prefab) -> Option<Entity> {
//         world_impl::instantiate(
//             &mut self.entities,
//             &mut self.scene,
//             &mut self.renderables,
//             &mut self.tags,
//             prefab,
//         ).ok()
//     }
// }

// #[test]
// fn find() {
//     let mut testbed = Testbed::new();

//     let e1 = testbed.create("room.obj");
//     let e2 = testbed.create("floor");
//     let e3 = testbed.create("tallBox");
//     let e4 = testbed.create("shortBox");

//     testbed.scene.set_parent(e2, e1, false).unwrap();
//     testbed.scene.set_parent(e3, e1, false).unwrap();
//     testbed.scene.set_parent(e4, e3, false).unwrap();

//     assert_eq!(testbed.find("room.obj"), Some(e1));
//     assert_eq!(testbed.find("room.obj/"), Some(e1));
//     assert_eq!(testbed.find("room.obj//"), Some(e1));
//     assert_eq!(testbed.find("/room.obj"), Some(e1));
//     assert_eq!(testbed.find("//room.obj"), Some(e1));
//     assert_eq!(testbed.find("/room.obj//"), Some(e1));

//     assert_eq!(testbed.find("room.obj/floor"), Some(e2));
//     assert_eq!(testbed.find("room.obj/tallBox"), Some(e3));
//     assert_eq!(testbed.find("room.obj/tallBox/shortBox"), Some(e4));

//     assert_eq!(testbed.find("room.obj/blahblah"), None);
// }

// #[test]
// fn instantiate() {
//     use crayon_3d::assets::prefab::PrefabNode;

//     let mut prefab = Prefab {
//         nodes: Vec::new(),
//         universe_meshes: Vec::new(),
//         meshes: Vec::new(),
//     };

//     prefab.nodes.push(PrefabNode {
//         name: "room.obj".into(),
//         local_transform: Transform::default(),
//         first_child: Some(1),
//         next_sib: None,
//         mesh_renderer: None,
//     });

//     prefab.nodes.push(PrefabNode {
//         name: "floor".into(),
//         local_transform: Transform::default(),
//         first_child: Some(2),
//         next_sib: None,
//         mesh_renderer: None,
//     });

//     prefab.nodes.push(PrefabNode {
//         name: "tallBox".into(),
//         local_transform: Transform::default(),
//         first_child: None,
//         next_sib: Some(3),
//         mesh_renderer: None,
//     });

//     prefab.nodes.push(PrefabNode {
//         name: "shortBox".into(),
//         local_transform: Transform::default(),
//         first_child: None,
//         next_sib: None,
//         mesh_renderer: None,
//     });

//     let mut testbed = Testbed::new();
//     let e1 = testbed.instantiate(&prefab).unwrap();

//     assert_eq!(testbed.entities.len(), 4);
//     assert_eq!(testbed.find("room.obj"), Some(e1));
//     assert!(testbed.find("room.obj/floor/tallBox").is_some());
// }
