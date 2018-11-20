use utils::prelude::Component;
use Entity;

use super::prelude::{Camera, Lit, MeshRenderer, Renderer};

pub struct HeadlessRenderer {
    materials: Component<()>,
}

impl HeadlessRenderer {
    pub fn new() -> Self {
        HeadlessRenderer {
            materials: Component::new(),
        }
    }
}

impl Renderer for HeadlessRenderer {
    type Mtl = ();

    fn add_mtl(&mut self, ent: Entity, mtl: Self::Mtl) {
        self.materials.add(ent, mtl);
    }

    fn mtl(&self, ent: Entity) -> Option<&Self::Mtl> {
        self.materials.get(ent)
    }

    fn mtl_mut(&mut self, ent: Entity) -> Option<&mut Self::Mtl> {
        self.materials.get_mut(ent)
    }

    fn remove_mtl(&mut self, ent: Entity) {
        self.materials.remove(ent);
    }

    fn submit(&mut self, _: &Camera, _: &[Lit], _: &[MeshRenderer]) {}
}
