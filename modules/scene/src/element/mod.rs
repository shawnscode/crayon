pub mod light;
pub mod camera;
pub mod mesh_renderer;

pub use self::light::{Light, LitSrc};
pub use self::camera::{Camera, Projection};
pub use self::mesh_renderer::MeshRenderer;

use crayon::ecs::{Component, VecArena};

/// The contrainer of elements that supported in `Scene`.
#[derive(Debug, Clone, Copy)]
pub enum Element {
    None,
    Light(Light),
    Camera(Camera),
    Mesh(MeshRenderer),
}

impl Component for Element {
    type Arena = VecArena<Element>;
}

impl Into<Element> for Light {
    fn into(self) -> Element {
        Element::Light(self)
    }
}

impl Into<Element> for Camera {
    fn into(self) -> Element {
        Element::Camera(self)
    }
}

impl Into<Element> for MeshRenderer {
    fn into(self) -> Element {
        Element::Mesh(self)
    }
}

impl Into<Element> for () {
    fn into(self) -> Element {
        Element::None
    }
}
