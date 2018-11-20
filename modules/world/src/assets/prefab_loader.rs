use std::io::Cursor;
use std::sync::Arc;

use crayon::errors::Result;
use crayon::res::utils::prelude::ResourceLoader;
use crayon::{bincode, video};

use super::prefab::*;

pub const MAGIC: [u8; 8] = [
    'P' as u8, 'R' as u8, 'E' as u8, 'B' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Clone)]
pub struct PrefabLoader {}

impl PrefabLoader {
    pub fn new() -> Self {
        PrefabLoader {}
    }
}

impl ResourceLoader for PrefabLoader {
    type Handle = PrefabHandle;
    type Intermediate = Prefab;
    type Resource = Arc<Prefab>;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        if &bytes[0..8] != &MAGIC[..] {
            bail!("[PrefabLoader] MAGIC number not match.");
        }

        let mut file = Cursor::new(&bytes[8..]);
        let mut prefab: Prefab = bincode::deserialize_from(&mut file)?;

        for &v in &prefab.universe_meshes {
            let mesh = video::create_mesh_from_uuid(v)?;
            prefab.meshes.push(mesh);
        }

        info!(
            "[PrefabLoader] load {:?}. (Nodes: {}, Meshes: {})",
            handle,
            prefab.nodes.len(),
            prefab.meshes.len()
        );

        Ok(prefab)
    }

    fn create(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Resource> {
        info!("[PrefabLoader] create {:?}.", handle);
        Ok(Arc::new(item))
    }

    fn delete(&self, handle: Self::Handle, prefab: Self::Resource) {
        info!("[PrefabLoader] delete {:?}.", handle);
        for &v in &prefab.meshes {
            video::delete_mesh(v);
        }
    }
}
