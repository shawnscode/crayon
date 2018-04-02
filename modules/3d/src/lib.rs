#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;

pub mod errors;
pub mod components;
pub mod graphics;
pub mod scene;
pub mod ent;
pub mod assets;
pub mod resources;

pub mod prelude {
    pub use scene::{Scene, SceneSetup};
    pub use assets::prelude::*;
    pub use components::prelude::*;
    pub use graphics::prelude::*;
    pub use crayon::ecs::world::Entity;
    pub use crayon::ecs::view::{ArenaGet, ArenaGetMut, Join};
    pub use ent::{EntReader, EntRef, EntRefMut, EntWriter};
}
