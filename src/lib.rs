#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;
extern crate bit_set;
extern crate deque;
extern crate rand;
extern crate libc;
extern crate zip;
extern crate json;
extern crate glutin;
extern crate gl;
extern crate cgmath;
#[macro_use]
extern crate approx;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
#[macro_use]
extern crate derive_builder;
extern crate image;

#[macro_use]
pub mod utility;
pub mod core;
pub mod multitask;
#[macro_use]
pub mod ecs;
pub mod resource;
#[macro_use]
pub mod graphics;
pub mod scene;

pub use core::Application;
pub use ecs::*;
pub use multitask::ThreadPool;

use cgmath as math;
use core::engine::Subsystem;

unsafe impl Send for World {}
unsafe impl Sync for World {}

impl Subsystem for World {}