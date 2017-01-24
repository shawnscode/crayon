#[macro_use]
extern crate lazy_static;
extern crate bit_set;
extern crate deque;
extern crate rand;
extern crate libc;

pub mod utility;

//
pub mod engine;
pub use self::engine::{Engine, Subsystem};

//
// pub mod application;
// pub use self::application::Application;

//
pub mod multitask;
pub use self::multitask::ThreadPool;

//
pub mod ecs;
pub use self::ecs::*;

unsafe impl Send for World {}
unsafe impl Sync for World {}

impl Subsystem for World {}