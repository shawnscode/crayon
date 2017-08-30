use resource;
use ecs::HashMapStorage;

pub struct Mesh {
    visible: bool,
    primitive: resource::PrimitiveItem,
    mat: resource::MaterialItem,
}

impl Mesh {
    pub fn new(primitive: resource::PrimitiveItem, mat: resource::MaterialItem) -> Self {
        Mesh {
            visible: true,
            primitive: primitive,
            mat: mat,
        }
    }

    pub fn material(&self) -> resource::MaterialItem {
        self.mat.clone()
    }

    pub fn primitive(&self) -> resource::PrimitiveItem {
        self.primitive.clone()
    }
}

impl super::Renderable for Mesh {
    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

declare_component!(Mesh, HashMapStorage);