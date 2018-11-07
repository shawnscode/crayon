// extern crate crayon;
// extern crate rand;

// use std::sync::mpsc;
// use std::sync::Arc;
// use std::thread;
// use std::time::Duration;

// use crayon::res::prelude::*;

// fn testbed() -> Arc<ResourceSystemShared> {
//     let dir = ::std::env::current_dir()
//         .unwrap()
//         .join("examples")
//         .join("resources");

//     let mut res = ResourceSystem::new().unwrap();
//     res.mount("res", Directory::new(dir).unwrap()).unwrap();
//     res.shared()
// }

// #[test]
// fn stress() {
//     let res = testbed();
//     let video = crayon::video::VideoSystem::headless(res).shared();

//     let (tx, rx) = mpsc::channel();
//     let mut handles = Vec::new();
//     for _ in 0..8 {
//         let video = video.clone();
//         let tx = tx.clone();
//         let t = thread::spawn(move || {
//             for _ in 0..(rand::random::<usize>() % 1024) {
//                 let handle = video.create_texture_from("res:crate.bmp").unwrap();
//                 video.delete_texture(handle);
//             }

//             tx.send(()).unwrap();
//         });

//         handles.push(t);
//     }

//     for _ in handles {
//         rx.recv_timeout(Duration::from_secs(5))
//             .expect("deadlock found!");
//     }
// }

// #[test]
// #[should_panic]
// fn from_unknown_vfs() {
//     let res = testbed();
//     let video = crayon::video::VideoSystem::headless(res).shared();

//     video.create_texture_from("unknown:crate.bmp").unwrap();
// }
