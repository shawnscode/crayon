use crayon::graphics::prelude::*;
use assets::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub index: MeshIndex,
    pub material: MaterialHandle,
}
