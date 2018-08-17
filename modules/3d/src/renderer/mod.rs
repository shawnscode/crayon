mod camera;
pub use self::camera::Camera;

mod lit;
pub use self::lit::{Lit, LitSource};

mod mesh_renderer;
pub use self::mesh_renderer::MeshRenderer;

pub mod simple;
pub use self::simple::SimpleRenderPipeline;

use crayon::ecs::prelude::*;
use std::collections::HashMap;

use scene::SceneGraph;

pub trait RenderPipeline {
    fn submit(&mut self, camera: &Camera, lits: &[Lit], meshes: &[MeshRenderer]);
}

pub struct Renderer {
    cameras: Renderable<Camera>,
    lits: Renderable<Lit>,
    meshes: Renderable<MeshRenderer>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            cameras: Renderable::new(),
            lits: Renderable::new(),
            meshes: Renderable::new(),
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

impl Renderer {
    pub fn draw(&mut self, pipeline: &mut RenderPipeline, scene: &SceneGraph) {
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

struct Renderable<T> {
    remap: HashMap<Entity, usize>,
    entities: Vec<Entity>,
    data: Vec<T>,
}

impl<T> Renderable<T> {
    #[inline]
    fn new() -> Self {
        Renderable {
            remap: HashMap::new(),
            entities: Vec::new(),
            data: Vec::new(),
        }
    }

    #[inline]
    fn add(&mut self, ent: Entity, v: T) {
        assert!(
            !self.remap.contains_key(&ent),
            "Ent already has components in Renderer."
        );

        self.remap.insert(ent, self.data.len());
        self.entities.push(ent);
        self.data.push(v);
    }

    #[inline]
    fn remove(&mut self, ent: Entity) {
        if let Some(v) = self.remap.remove(&ent) {
            self.entities.swap_remove(v);
            self.data.swap_remove(v);

            if self.remap.len() > 0 {
                *self.remap.get_mut(&self.entities[v]).unwrap() = v;
            }
        }
    }

    #[inline]
    fn get(&self, ent: Entity) -> Option<&T> {
        let data = &self.data;
        self.remap.get(&ent).map(|&index| &data[index])
    }

    #[inline]
    fn get_mut(&mut self, ent: Entity) -> Option<&mut T> {
        let data = &mut self.data;
        self.remap.get(&ent).map(move |&index| &mut data[index])
    }
}
