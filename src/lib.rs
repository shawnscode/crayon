//! # Crayon Game Engine
//!
//!

extern crate libc;
extern crate glutin;
extern crate gl;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate error_chain;

extern crate zip;
extern crate two_lock_queue;
extern crate bit_set;

#[macro_use]
pub extern crate lazy_static;
pub extern crate rayon;
pub extern crate cgmath as math;

#[macro_use]
pub mod utils;
pub mod application;
#[macro_use]
pub mod ecs;
#[macro_use]
pub mod graphics;
pub mod resource;
pub mod input;
pub mod prelude;