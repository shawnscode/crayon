#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;
extern crate bit_set;
extern crate deque;
extern crate rand;
extern crate libc;
extern crate zip;
extern crate glutin;
extern crate gl;
extern crate cgmath;
extern crate uuid;

#[macro_use]
extern crate approx;
extern crate byteorder;
#[macro_use]
extern crate derive_builder;
extern crate image;
extern crate rayon;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate json;
extern crate bincode;

#[macro_use]
pub mod utility;
pub mod core;
#[macro_use]
pub mod ecs;
pub mod resource;
#[macro_use]
pub mod graphics;
pub mod scene;

pub use core::Application;
pub use ecs::*;

use cgmath as math;
use core::engine::Subsystem;

unsafe impl Send for World {}
unsafe impl Sync for World {}

impl Subsystem for World {}