mod camera;
pub use self::camera::Camera;

mod lit;
pub use self::lit::{Lit, LitSource};

mod mesh_renderer;
pub use self::mesh_renderer::MeshRenderer;

pub mod simple;
pub use self::simple::{SimpleMaterial, SimpleRenderer};

use scene::SceneGraph;
use {Component, Entity};

pub trait Renderer {
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
    pub fn add_mesh(&mut self, ent: Entity, mesh: MeshRenderer) {
        self.meshes.add(ent, mesh);
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
    pub fn draw(&mut self, pipeline: &mut Renderer, scene: &SceneGraph) {
        for (i, v) in self.cameras.data.iter_mut().enumerate() {
            if let Some(transform) = scene.transform(self.cameras.entities[i]) {
                v.transform = transform;
            }
        }

        for (i, v) in self.lits.data.iter_mut().enumerate() {
            if let Some(transform) = scene.transform(self.lits.entities[i]) {
                v.transform = transform;
            }
        }

        for (i, v) in self.meshes.data.iter_mut().enumerate() {
            if let Some(transform) = scene.transform(self.meshes.entities[i]) {
                v.transform = transform;
                v.ent = self.meshes.entities[i];
            }
        }

        for v in &self.cameras.data {
            pipeline.submit(&v, &self.lits.data, &self.meshes.data);
        }
    }
}
