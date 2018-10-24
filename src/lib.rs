//! # What is This?
//! Crayon is a small, portable and extensible game framework, which loosely inspired by some
//! amazing blogs on [bitsquid](https://bitsquid.blogspot.de), [molecular](https://blog.molecular-matters.com)
//! and [floooh](http://floooh.github.io/).
//!
//! Some goals include:
//!
//! - Extensible through external code modules;
//! - Run on macOS, Linux, Windows, iOS, Android from the same source;
//! - Built from the ground up to focus on multi-thread friendly with a work-stealing job scheduler;
//! - Stateless, layered, multithread render system with OpenGL(ES) 3.0 backends;
//! - Simplified assets workflow and asynchronous data loading from various filesystem;
//! - Unified interfaces for handling input devices across platforms;
//! - etc.
//!
//! This project adheres to [Semantic Versioning](http://semver.org/), all notable changes will be documented in this [file](./CHANGELOG.md).
//!
//! ### Quick Example
//! For the sake of brevity, you can als run a simple and quick example with commands:
//!
//! ``` sh
//! git clone git@github.com:shawnscode/crayon.git
//! cargo run --example modules_3d_prefab
//! ```

extern crate crossbeam_deque;
#[macro_use]
extern crate cgmath;
extern crate gl;
extern crate glutin;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;
extern crate byteorder;
extern crate serde_json;

pub extern crate bincode;
pub extern crate uuid;

#[doc(hidden)]
pub use cgmath::*;
#[doc(hidden)]
pub use log::*;

// FIXME: unresolved serde proc-macro re-export. https://github.com/serde-rs/serde/issues/1147
// #[doc(hidden)]
// pub use serde::*;
// FIXME: unresolved failure proc-macro re-export.
// #[doc(hidden)]
// pub use failure::*;

#[macro_use]
pub mod errors;
#[macro_use]
pub mod utils;
pub mod application;
#[macro_use]
pub mod video;
pub mod input;
pub mod math;
pub mod prelude;
pub mod res;
pub mod sched;
