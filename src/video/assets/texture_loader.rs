use std::io::Read;
use std::sync::Arc;

use math;
use res::byteorder::ByteOrderRead;
use res::errors::*;

use super::super::VideoSystemShared;
use super::texture::*;

pub const MAGIC: [u8; 7] = ['C' as u8, 'T' as u8, 'E' as u8, 'X' as u8, ' ' as u8, 0, 1];

pub struct TextureLoader {
    buf: Vec<u8>,
    video: Arc<VideoSystemShared>,
}

impl TextureLoader {
    pub fn load<T>(&mut self, mut file: T) -> Result<Texture>
    where
        T: Read,
    {
        let mut buf = [0; 8];
        file.read_exact(&mut buf[0..7])?;

        // MAGIC: [u8; 7]
        if &buf[0..7] != &MAGIC[..] {
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
        self.buf.clear();
        file.read_to_end(&mut self.buf)?;

        let tex = self.video.create_texture(params, self.buf.as_slice())?;
        Ok(tex)
    }
}
