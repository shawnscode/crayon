#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;

pub mod assets;
pub mod components;
pub mod ent;
pub mod errors;
pub mod graphics;
pub mod resources;
pub mod scene;

pub mod prelude {
    pub use assets::prelude::*;
    pub use components::prelude::*;
    pub use crayon::ecs::view::{ArenaGet, ArenaGetMut, Join};
    pub use crayon::ecs::world::Entity;
    pub use ent::{EntReader, EntRef, EntRefMut, EntWriter};
    pub use graphics::prelude::*;
    pub use scene::{Scene, SceneSetup};
}
