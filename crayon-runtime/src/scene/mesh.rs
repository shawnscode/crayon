use resource;
use ecs::HashMapStorage;

pub struct Mesh {
    visible: bool,
    primitive: resource::PrimitivePtr,
    mat: resource::MaterialPtr,
}

impl Mesh {
    pub fn new(primitive: resource::PrimitivePtr, mat: resource::MaterialPtr) -> Self {
        Mesh {
            visible: true,
            primitive: primitive,
            mat: mat,
        }
    }

    pub fn material(&self) -> resource::MaterialPtr {
        self.mat.clone()
    }

    pub fn primitive(&self) -> resource::PrimitivePtr {
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