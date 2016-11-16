pub mod utils;
pub mod ecs;
pub mod engine;

pub use self::engine::Subsystem;
pub use self::ecs::World;

impl Subsystem for World {}

