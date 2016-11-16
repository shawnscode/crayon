//! Entity Component System (ECS)

pub mod component;
pub mod world;

pub use self::world::World;
use super::utils::handle::*;

pub type Entity = Handle;
