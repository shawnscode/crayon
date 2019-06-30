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

#![allow(clippy::new_ret_no_self)]

#[cfg(not(target_arch = "wasm32"))]
extern crate gl;
#[cfg(not(target_arch = "wasm32"))]
extern crate glutin;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;
#[cfg(target_arch = "wasm32")]
extern crate js_sys;
#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;
#[cfg(target_arch = "wasm32")]
extern crate web_sys;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
extern crate cgmath;
#[macro_use]
extern crate serde;
extern crate byteorder;
extern crate serde_json;

extern crate crossbeam_deque;
extern crate inlinable_string;
extern crate smallvec;

pub extern crate bincode;
pub extern crate uuid;

pub use cgmath::{assert_relative_eq, assert_relative_ne, assert_ulps_eq, assert_ulps_ne};
pub use cgmath::{relative_eq, relative_ne, ulps_eq, ulps_ne};
pub use log::{error, info, warn};

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
pub mod window;
pub mod network;

#[macro_export]
macro_rules! main {
    ($codes: block) => {
        #[cfg(target_arch = "wasm32")]
        extern crate wasm_bindgen;
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen::prelude::wasm_bindgen;

        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen(start)]
        pub fn run() {
            $codes
        }

        fn main() {
            #[cfg(not(target_arch = "wasm32"))]
            $codes
        }
    };
}
