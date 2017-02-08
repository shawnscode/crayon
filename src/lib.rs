#[macro_use]
extern crate lazy_static;
extern crate bit_set;
extern crate deque;
extern crate rand;
extern crate libc;
extern crate zip;
extern crate json;
extern crate glutin;
extern crate gl;
extern crate cgmath;

// use cgmath as math;

pub mod utility;
pub mod core;
pub mod multitask;
pub mod ecs;
pub mod resource;
pub mod graphics;

pub use core::Application;
pub use ecs::*;
pub use multitask::ThreadPool;

use core::engine::Subsystem;

unsafe impl Send for World {}
unsafe impl Sync for World {}

impl Subsystem for World {}