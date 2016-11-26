#[macro_use]
extern crate lazy_static;

pub mod utils;
pub mod ecs;
pub mod engine;

pub use self::engine::Subsystem;
pub use self::ecs::World;
pub use self::ecs::component::{Component, HashMapStorage};

unsafe impl Send for World {}
unsafe impl Sync for World {}
impl Subsystem for World {}
