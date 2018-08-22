#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;
extern crate bincode;
extern crate uuid;

pub mod assets;
pub mod renderer;
pub mod scene;
pub mod standard;

pub mod prelude {
    pub use renderer::{Camera, Lit, MeshRenderer, Renderer, SimpleRenderPipeline};
    pub use scene::SceneGraph;
    pub use standard::Standard;
}
