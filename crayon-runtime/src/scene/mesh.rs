use resource;
use ecs::HashMapStorage;

pub struct Mesh {
    visible: bool,
    primitive: resource::PrimitivePtr,
    mat: Option<resource::MaterialPtr>,
}

impl Mesh {
    pub fn new(primitive: resource::PrimitivePtr, mat: Option<resource::MaterialPtr>) -> Self {
        Mesh {
            visible: true,
            primitive: primitive,
            mat: mat,
        }
    }

    pub fn primitive(&self) -> &resource::PrimitivePtr {
        &self.primitive
    }
}

impl super::Renderable for Mesh {
    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn material(&self) -> Option<&resource::MaterialPtr> {
        self.mat.as_ref()
    }
}

declare_component!(Mesh, HashMapStorage);