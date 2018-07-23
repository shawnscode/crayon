use std::io::Read;
use std::sync::Arc;

use math;
use res::byteorder::ByteOrderRead;
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
    const SCHEMA: &'static str = "vtex";
    type Handle = TextureHandle;

    fn create(&self) -> Result<Self::Handle> {
        let handle = self.video.loader_create_texture()?;
        Ok(handle)
    }

    fn load(&self, handle: Self::Handle, file: &mut dyn Read) -> Result<()> {
        let mut buf = [0; 8];
        file.read_exact(&mut buf[0..8])?;

        // MAGIC: [u8; 8]
        if &buf[0..8] != &MAGIC[..] {
            return Err(Error::Malformed(
                "[TextureLoader] MAGIC number not match.".into(),
            ));
        }

        let mut params = TextureParams::default();

        //
        file.read_exact(&mut buf[0..4])?;

        params.wrap = unsafe { ::std::mem::transmute_copy(&buf[0]) };
        params.filter = unsafe { ::std::mem::transmute_copy(&buf[1]) };
        params.mipmap = buf[2] > 0;
        params.format = unsafe { ::std::mem::transmute_copy(&buf[3]) };
        params.dimensions = math::Vector2::new(file.read_u32()?, file.read_u32()?);

        //
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let slice = buf.as_slice();
        self.video.loader_update_texture(handle, params, slice)?;
        Ok(())
    }

    fn delete(&self, handle: Self::Handle) -> Result<()> {
        self.video.delete_texture(handle);
        Ok(())
    }
}
