use bincode;
use std::io::Read;
use std::sync::Arc;

use res::errors::*;

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
        let handle = self.video.loader_create_texture()?;
        Ok(handle)
    }

    fn load(&self, handle: Self::Handle, mut file: &mut dyn Read) -> Result<()> {
        let mut buf = [0; 8];
        file.read_exact(&mut buf[0..8])?;

        // MAGIC: [u8; 8]
        if &buf[0..8] != &MAGIC[..] {
            return Err(Error::Malformed(
                "[TextureLoader] MAGIC number not match.".into(),
            ));
        }

        let params = bincode::deserialize_from(&mut file)?;
        let data = bincode::deserialize_from(&mut file)?;

        self.video.loader_update_texture(handle, params, data)?;
        Ok(())
    }

    fn delete(&self, handle: Self::Handle) -> Result<()> {
        self.video.delete_texture(handle);
        Ok(())
    }
}
