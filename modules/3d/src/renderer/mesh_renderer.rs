use crayon::ecs::prelude::*;
use crayon::video::prelude::*;

use scene::Transform;

#[derive(Debug, Clone, Copy)]
pub struct MeshRenderer {
    /// The mesh handle used by the renderer.
    pub mesh: MeshHandle,
    /// Indicates whether this object cast shadows.
    pub shadow_caster: bool,
    /// Indicates whether this object receive shadows.
    pub shadow_receiver: bool,
    /// Is this renderer visible.
    pub visible: bool,

    #[doc(hidden)]
    pub transform: Transform,
    #[doc(hidden)]
    pub ent: Entity,
}

impl Default for MeshRenderer {
    fn default() -> Self {
        MeshRenderer {
            mesh: MeshHandle::default(),
            shadow_caster: false,
            shadow_receiver: false,
            visible: true,
            transform: Transform::default(),
            ent: Entity::default(),
        }
    }
}
