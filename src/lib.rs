#[macro_use]
extern crate lazy_static;
extern crate bit_set;
extern crate deque;
extern crate rand;

pub mod utility;
pub mod ecs;
pub mod engine;
pub mod multitask;

pub use self::engine::{Engine, Subsystem};
pub use self::ecs::World;
pub use self::ecs::component::{Component, HashMapStorage};

unsafe impl Send for World {}
unsafe impl Sync for World {}
impl Subsystem for World {}
