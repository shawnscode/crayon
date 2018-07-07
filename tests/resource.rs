// extern crate crayon;

// use crayon::prelude::*;
// // use std::any::TypeId;

// #[derive(Debug)]
// struct Text {
//     pub value: String,
// }

// // impl Text {
// //     fn id() -> TypeId {
// //         TypeId::of::<Text>()
// //     }
// // }

// impl resource::Resource for Text {
//     fn size(&self) -> usize {
//         self.value.len()
//     }
// }

// impl resource::ResourceParser for Text {
//     type Item = Text;

//     fn parse(bytes: &[u8]) -> resource::errors::Result<Self::Item> {
//         Ok(Text { value: String::from_utf8_lossy(&bytes).into_owned() })
//     }
// }

// #[test]
// fn load() {
//     let sys = ResourceSystem::new().unwrap();

//     let fs = resource::filesystem::DirectoryFS::new("tests/resources").unwrap();
//     sys.mount("res", fs).unwrap();

//     {
//         let future = sys.shared().load::<Text, &str>("/res/mock.txt");
//         let text = future.wait().unwrap();
//         assert_eq!(text.value, "Hello, World!");

//         sys.advance().unwrap();

//         {
//             // let info = info.arenas.get(&Text::id()).unwrap();
//             // assert_eq!(info.size, "Hello, World!".len());
//             // assert_eq!(info.num, 1);
//         }

//         // No duplicated copys.
//         let future = sys.shared().load::<Text, &str>("/res/mock.txt");
//         let t2 = future.wait().unwrap();
//         assert_eq!(t2.value, "Hello, World!");

//         sys.advance().unwrap();

//         {
//             // let info = info.arenas.get(&Text::id()).unwrap();
//             // assert_eq!(info.size, "Hello, World!".len());
//             // assert_eq!(info.num, 1);
//         }
//     }

//     // Free all the resources which has no external references.
//     sys.advance().unwrap();

//     {
//         // let info = info.arenas.get(&Text::id()).unwrap();
//         // assert_eq!(info.size, 0);
//         // assert_eq!(info.num, 0);
//     }
// }
