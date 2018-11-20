mod camera;
mod lit;
mod mesh_renderer;
mod simple;

pub mod headless;

pub mod prelude {
    pub use super::camera::Camera;
    pub use super::lit::{Lit, LitSource};
    pub use super::mesh_renderer::MeshRenderer;
    pub use super::simple::{SimpleMaterial, SimpleRenderer};
    pub use super::{Renderable, Renderer};
}

use spatial::prelude::SceneGraph;
use utils::prelude::Component;
use Entity;

use self::camera::Camera;
use self::lit::{Lit, LitSource};
use self::mesh_renderer::MeshRenderer;

pub trait Renderer {
    type Mtl;

    fn add_mtl(&mut self, ent: Entity, mtl: Self::Mtl);
    fn mtl(&self, ent: Entity) -> Option<&Self::Mtl>;
    fn mtl_mut(&mut self, ent: Entity) -> Option<&mut Self::Mtl>;
    fn remove_mtl(&mut self, ent: Entity);

    fn submit(&mut self, camera: &Camera, lits: &[Lit], meshes: &[MeshRenderer]);
}

pub struct Renderable {
    cameras: Component<Camera>,
    lits: Component<Lit>,
    meshes: Component<MeshRenderer>,
}

impl Renderable {
    pub fn new() -> Self {
        Renderable {
            cameras: Component::new(),
            lits: Component::new(),
            meshes: Component::new(),
        }
    }

    #[inline]
    pub fn add_camera(&mut self, ent: Entity, camera: Camera) {
        self.cameras.add(ent, camera);
    }

    #[inline]
    pub fn camera(&self, ent: Entity) -> Option<&Camera> {
        self.cameras.get(ent)
    }

    #[inline]
    pub fn camera_mut(&mut self, ent: Entity) -> Option<&mut Camera> {
        self.cameras.get_mut(ent)
    }

    #[inline]
    pub fn remove_camera(&mut self, ent: Entity) {
        self.cameras.remove(ent);
    }

    #[inline]
    pub fn add_lit(&mut self, ent: Entity, lit: Lit) {
        self.lits.add(ent, lit);
    }

    #[inline]
    pub fn lit(&self, ent: Entity) -> Option<&Lit> {
        self.lits.get(ent)
    }

    #[inline]
    pub fn lit_mut(&mut self, ent: Entity) -> Option<&mut Lit> {
        self.lits.get_mut(ent)
    }

    #[inline]
    pub fn remove_lit(&mut self, ent: Entity) {
        self.lits.remove(ent);
    }

    #[inline]
    pub fn add_mesh<T: Into<MeshRenderer>>(&mut self, ent: Entity, mesh: T) {
        self.meshes.add(ent, mesh.into());
    }

    #[inline]
    pub fn mesh(&self, ent: Entity) -> Option<&MeshRenderer> {
        self.meshes.get(ent)
    }

    #[inline]
    pub fn mesh_mut(&mut self, ent: Entity) -> Option<&mut MeshRenderer> {
        self.meshes.get_mut(ent)
    }

    #[inline]
    pub fn remove_mesh(&mut self, ent: Entity) {
        self.meshes.remove(ent);
    }
}

impl Renderable {
    pub fn draw<R: Renderer>(&mut self, renderer: &mut R, sg: &SceneGraph) {
        for (i, v) in self.cameras.data.iter_mut().enumerate() {
            if let Some(transform) = sg.transform(self.cameras.entities[i]) {
                v.transform = transform;
            }
        }

        for (i, v) in self.lits.data.iter_mut().enumerate() {
            if let Some(transform) = sg.transform(self.lits.entities[i]) {
                v.transform = transform;
            }
        }

        for (i, v) in self.meshes.data.iter_mut().enumerate() {
            if let Some(transform) = sg.transform(self.meshes.entities[i]) {
                v.transform = transform;
                v.ent = self.meshes.entities[i];
            }
        }

        for v in &self.cameras.data {
            renderer.submit(&v, &self.lits.data, &self.meshes.data);
        }
    }
}
