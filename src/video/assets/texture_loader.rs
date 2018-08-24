use bincode;
use std::io::Read;
use std::sync::Arc;

use errors::*;

use super::super::VideoSystemShared;
use super::texture::*;

pub const MAGIC: [u8; 8] = [
    'V' as u8, 'T' as u8, 'E' as u8, 'X' as u8, ' ' as u8, 0, 0, 1,
];

pub struct TextureLoader {
    video: Arc<VideoSystemShared>,
}

impl TextureLoader {
    pub fn new(video: Arc<VideoSystemShared>) -> Self {
        TextureLoader { video: video }
    }
}

impl ::res::ResourceHandle for TextureHandle {
    type Loader = TextureLoader;
}

impl ::res::ResourceLoader for TextureLoader {
    type Handle = TextureHandle;

    fn create(&self) -> Result<Self::Handle> {
        let handle = self.video.create_texture_async()?;
        info!("[TextureLoader] creates {:?}.", handle);
        Ok(handle)
    }

    fn load(&self, handle: Self::Handle, mut file: &mut dyn Read) -> Result<()> {
        let mut buf = [0; 8];
        file.read_exact(&mut buf[0..8])?;

        // magic: [u8; 8]
        if &buf[0..8] != &MAGIC[..] {
            bail!("[TextureLoader] MAGIC number not match.");
        }

        let params: TextureParams = bincode::deserialize_from(&mut file)?;
        let data = bincode::deserialize_from(&mut file)?;

        info!(
            "[TextureLoader] loads {:?} ({}x{} - {:?}).",
            handle, params.dimensions.x, params.dimensions.y, params.format
        );

        self.video.update_texture_async(handle, params, data)?;
        Ok(())
    }

    fn delete(&self, handle: Self::Handle) -> Result<()> {
        self.video.delete_texture(handle);
        info!("[TextureLoader] deletes {:?}.", handle);
        Ok(())
    }
}
