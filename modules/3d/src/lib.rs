#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;

pub mod assets;
pub mod renderers;
pub mod scene;
pub mod tags;

mod component;
use self::component::Component;

mod world;
pub use self::world::{world_impl, Entity, World};

pub mod prelude {
    pub use assets::WorldResources;
    pub use renderers::{Camera, Lit, MeshRenderer, SimpleRenderer};
    pub use scene::SceneGraph;
    pub use world::{Entity, World};
}
