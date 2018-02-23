use crayon::graphics::{MeshHandle, MeshIndex};

use assets::MaterialHandle;

#[derive(Debug, Copy, Clone)]
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub index: MeshIndex,
    pub material: MaterialHandle,
}
