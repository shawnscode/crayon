use std::io::Cursor;
use std::sync::Arc;

use crayon::application::prelude::*;
use crayon::bincode;
use crayon::errors::*;
use crayon::res::prelude::*;
use crayon::video::prelude::*;

use super::prefab::*;

pub const MAGIC: [u8; 8] = [
    'P' as u8, 'R' as u8, 'E' as u8, 'B' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Clone)]
pub struct PrefabLoader {
    video: Arc<VideoSystemShared>,
    res: Arc<ResourceSystemShared>,
}

impl PrefabLoader {
    pub fn new(ctx: &Context) -> Self {
        PrefabLoader {
            res: ctx.res.clone(),
            video: ctx.video.clone(),
        }
    }
}

impl Register for PrefabLoader {
    type Handle = PrefabHandle;
    type Intermediate = Prefab;
    type Value = Arc<Prefab>;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        if &bytes[0..8] != &MAGIC[..] {
            bail!("[PrefabLoader] MAGIC number not match.");
        }

        let mut file = Cursor::new(&bytes[8..]);
        let mut prefab: Prefab = bincode::deserialize_from(&mut file)?;

        for &v in &prefab.universe_meshes {
            let mesh = self.video.create_mesh_from_uuid(v)?;
            prefab.meshes.push(mesh);
        }

        // for &v in &prefab.meshes {
        //     self.res.wait(v)?;
        // }

        info!(
            "[PrefabLoader] loads {:?}. (Nodes: {}, Meshes: {})",
            handle,
            prefab.nodes.len(),
            prefab.meshes.len()
        );

        Ok(prefab)
    }

    fn attach(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Value> {
        info!("[PrefabLoader] attach {:?}.", handle);
        Ok(Arc::new(item))
    }

    fn detach(&self, handle: Self::Handle, prefab: Self::Value) {
        info!("[PrefabLoader] detach {:?}.", handle);

        for &v in &prefab.meshes {
            self.video.delete_mesh(v);
        }
    }
}
