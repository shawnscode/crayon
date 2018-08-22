#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;
extern crate bincode;
extern crate uuid;

pub mod assets;
pub mod renderers;
pub mod scene;

mod world;
pub use self::world::{Entity, World};

pub mod prelude {
    pub use assets::WorldResources;
    pub use renderers::{Camera, Lit, MeshRenderer, Renderer, SimpleRenderPipeline};
    pub use scene::SceneGraph;
    pub use world::{Entity, World};
}
