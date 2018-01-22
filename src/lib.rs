//! # What is This?
//!
//! Crayon is an experimental purpose game engine, written with a minimalistic
//! modular design philosophy. Its built from the ground up to focus on cache
//! friendly data layouts in multicore environments with entity-component based
//! architecture.
//!
//! It is loosely inspired by some amazing blogs on [bitsquid](https://bitsquid.blogspot.de)
//! and [molecular](https://blog.molecular-matters.com). Some goals include:
//!
//! - Extensible through external code modules;
//! - Run on [x]macOS, [x]Windows, iOS, Android, WebAssembly from the same source;
//! - Stateless, layered, multithread render system with OpenGL(ES) 2.0+ backends;
//! - Entity component system with a data-driven designs;
//! - Unified access to input devices across platforms;
//! - Asynchronous data loading from various filesystem.
//!
//! Please read the documents under modules for specific usages.
//!
//! ## Quick Example
//!
//! For the sake of brevity, you can also run a simple and quick example with commands:
//!
//! ```sh
//! git clone git@github.com:shawnscode/crayon.git && cd crayon/crayon-examples
//! cargo run imgui
//! ```

extern crate gl;
extern crate glutin;
extern crate libc;
pub extern crate cgmath as math;

#[macro_use]
extern crate error_chain;

extern crate two_lock_queue;
extern crate zip;

#[macro_use]
pub mod utils;
pub mod application;
#[macro_use]
pub mod ecs;
#[macro_use]
pub mod graphics;
pub mod resource;
pub mod input;
pub mod scene;
pub mod prelude;
