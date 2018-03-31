use crayon::graphics::prelude::*;
use crayon::ecs::prelude::*;
use assets::prelude::*;

#[derive(Debug, Clone)]
pub struct MeshRenderer {
    /// The mesh handle used by the renderer.
    pub mesh: MeshHandle,
    /// This is an array of all materials used by the renderer.
    pub materials: Vec<MaterialHandle>,
    /// Indicates whether this object cast shadows.
    pub shadow_caster: bool,
    /// Indicates whether this object receive shadows.
    pub shadow_receiver: bool,
    /// Is this renderer visible.
    pub visible: bool,
}

impl Component for MeshRenderer {
    type Arena = HashMapArena<MeshRenderer>;
}
