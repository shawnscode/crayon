use bincode;
use std::io::Read;
use std::sync::Arc;

use errors::*;

use super::super::VideoSystemShared;
use super::mesh::*;

pub const MAGIC: [u8; 8] = [
    'V' as u8, 'M' as u8, 'S' as u8, 'H' as u8, ' ' as u8, 0, 0, 1,
];

pub struct MeshLoader {
    video: Arc<VideoSystemShared>,
}

impl MeshLoader {
    pub fn new(video: Arc<VideoSystemShared>) -> Self {
        MeshLoader { video: video }
    }
}

impl ::res::ResourceHandle for MeshHandle {
    type Loader = MeshLoader;
}

impl ::res::ResourceLoader for MeshLoader {
    type Handle = MeshHandle;

    fn create(&self) -> Result<Self::Handle> {
        let handle = self.video.create_mesh_async()?;
        Ok(handle)
    }

    fn load(&self, handle: Self::Handle, mut file: &mut dyn Read) -> Result<()> {
        let mut buf = [0; 8];
        file.read_exact(&mut buf[0..8])?;

        // MAGIC: [u8; 8]
        if &buf[0..8] != &MAGIC[..] {
            bail!("[MeshLoader] MAGIC number not match.");
        }

        let params = bincode::deserialize_from(&mut file)?;
        let data = bincode::deserialize_from(&mut file)?;

        self.video.update_mesh_async(handle, params, data)?;
        Ok(())
    }

    fn delete(&self, handle: Self::Handle) -> Result<()> {
        self.video.delete_mesh(handle);
        Ok(())
    }
}
