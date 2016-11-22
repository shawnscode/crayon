//! Entity Component System (ECS)

pub mod component;
pub mod world;

pub use self::world::World;
pub use self::component::{Component, ComponentStorage, HashMapStorage};

use super::utils::handle::*;
pub type Entity = Handle;
