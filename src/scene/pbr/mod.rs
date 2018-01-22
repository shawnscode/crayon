//! Physically based rendering.

pub mod renderer;
pub mod scene;

pub use self::scene::PbrScene;
pub use self::renderer::{PbrMaterial, PbrMesh};
