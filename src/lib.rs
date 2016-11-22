pub mod utils;
pub mod ecs;
pub mod engine;

pub use self::engine::Subsystem;
pub use self::ecs::World;

unsafe impl Send for World {}
unsafe impl Sync for World {}
impl Subsystem for World {}
