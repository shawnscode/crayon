use graphics::{MeshHandle, MeshIndex};
use scene::MaterialHandle;

#[derive(Debug, Copy, Clone)]
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub index: MeshIndex,
    pub material: MaterialHandle,
}
