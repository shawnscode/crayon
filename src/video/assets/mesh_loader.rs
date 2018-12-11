use bincode;
use std::io::Cursor;
use std::sync::Arc;

use crate::errors::*;
use crate::res::utils::prelude::ResourceLoader;
use crate::utils::double_buf::DoubleBuf;

use super::super::backends::frame::{Command, Frame};
use super::mesh::*;

pub const MAGIC: [u8; 8] = [b'V', b'M', b'S', b'H', b' ', 0, 0, 1];

#[derive(Clone)]
pub struct MeshLoader {
    frames: Arc<DoubleBuf<Frame>>,
}

impl MeshLoader {
    pub(crate) fn new(frames: Arc<DoubleBuf<Frame>>) -> Self {
        MeshLoader { frames }
    }
}

impl ResourceLoader for MeshLoader {
    type Handle = MeshHandle;
    type Intermediate = (MeshParams, Option<MeshData>);
    type Resource = MeshParams;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        if bytes[0..8] != MAGIC[..] {
            bail!("[MeshLoader] MAGIC number not match.");
        }

        let mut file = Cursor::new(&bytes[8..]);
        let params: MeshParams = bincode::deserialize_from(&mut file)?;
        let data = bincode::deserialize_from(&mut file)?;

        info!(
            "[MeshLoader] load {:?}. (Verts: {}, Indxes: {})",
            handle, params.num_verts, params.num_idxes
        );

        Ok((params, Some(data)))
    }

    fn create(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Resource> {
        info!("[MeshLoader] create {:?}.", handle);
        item.0.validate(item.1.as_ref())?;
        let cmd = Command::CreateMesh(Box::new((handle, item.0.clone(), item.1)));
        self.frames.write().cmds.push(cmd);
        Ok(item.0)
    }

    fn delete(&self, handle: Self::Handle, _: Self::Resource) {
        info!("[MeshLoader] delete {:?}.", handle);
        let cmd = Command::DeleteMesh(handle);
        self.frames.write().cmds.push(cmd);
    }
}
