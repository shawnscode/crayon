#[macro_use]
extern crate lazy_static;
extern crate bit_set;
extern crate deque;
extern crate rand;
extern crate libc;
extern crate zip;
extern crate json;

pub mod utility;
pub mod core;
pub mod multitask;
pub mod ecs;
pub mod resource;

pub use core::engine::{Engine, Subsystem};
pub use self::multitask::ThreadPool;
pub use self::ecs::*;

unsafe impl Send for World {}
unsafe impl Sync for World {}

impl Subsystem for World {}