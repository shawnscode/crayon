#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;

pub mod errors;
pub mod node;
pub mod transform;
pub mod element;
pub mod scene;
pub mod renderer;
pub mod assets;

pub mod prelude {
    pub use node::Node;
    pub use transform::Transform;
    pub use scene::Scene;
    pub use assets::prelude::*;
    pub use element::prelude::*;
    pub use crayon::ecs::Entity;
    pub use crayon::ecs::world::{Arena, ArenaMut};
}
