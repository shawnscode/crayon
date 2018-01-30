mod uniforms;
pub use self::uniforms::RenderUniform;

mod graph;
pub use self::graph::RenderGraph;

use graphics::{MeshHandle, MeshIndex};
use scene::MaterialHandle;

#[derive(Debug, Copy, Clone)]
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub index: MeshIndex,
    pub material: MaterialHandle,
}
