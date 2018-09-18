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

mod world_resources;
pub use self::world_resources::{WorldResources, WorldResourcesShared};

pub mod prelude {
    pub use assets::Prefab;
    pub use renderers::{Camera, Lit, MeshRenderer, SimpleMaterial, SimpleRenderer};
    pub use scene::{SceneGraph, Transform};
    pub use world::{Entity, World};
    pub use world_resources::{WorldResources, WorldResourcesShared};
}
