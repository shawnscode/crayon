use crayon::errors::*;
use crayon::uuid::Uuid;
use crayon::video::assets::mesh::MeshHandle;

use scene::Transform;

impl_handle!(PrefabHandle);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prefab {
    ///
    pub nodes: Vec<PrefabNode>,
    pub universe_meshes: Vec<Uuid>,

    #[serde(skip)]
    pub meshes: Vec<MeshHandle>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrefabNode {
    /// The name of this node.
    pub name: String,
    /// The transformation in local space.
    pub local_transform: Transform,
    /// The first child index of this node.
    pub first_child: Option<usize>,
    /// The sibling index of this node.
    pub next_sib: Option<usize>,
    /// The optional mesh renderer.
    pub mesh_renderer: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PrefabMeshRenderer {
    /// The mesh index.
    pub mesh: usize,
    /// Indicates whether this object cast shadows.
    pub shadow_caster: bool,
    /// Indicates whether this object receive shadows.
    pub shadow_receiver: bool,
    /// Is this renderer visible.
    pub visible: bool,
}

impl Prefab {
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
