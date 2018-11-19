use bincode;
use std::io::Cursor;
use std::sync::Arc;

use errors::*;
use res::utils::prelude::ResourceLoader;
use utils::double_buf::DoubleBuf;

use super::super::backends::frame::{Command, Frame};
use super::texture::*;

pub const MAGIC: [u8; 8] = [
    'V' as u8, 'T' as u8, 'E' as u8, 'X' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Clone)]
pub struct TextureLoader {
    frames: Arc<DoubleBuf<Frame>>,
}

impl TextureLoader {
    pub(crate) fn new(frames: Arc<DoubleBuf<Frame>>) -> Self {
        TextureLoader { frames: frames }
    }
}

impl ResourceLoader for TextureLoader {
    type Handle = TextureHandle;
    type Intermediate = (TextureParams, Option<TextureData>);
    type Resource = TextureParams;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        if &bytes[0..8] != &MAGIC[..] {
            bail!("[TextureLoader] MAGIC number not match.");
        }

        let mut file = Cursor::new(&bytes[8..]);
        let params: TextureParams = bincode::deserialize_from(&mut file)?;
        let data = bincode::deserialize_from(&mut file)?;

        info!(
            "[TextureLoader] loads {:?} ({}x{} - {:?}).",
            handle, params.dimensions.x, params.dimensions.y, params.format
        );

        Ok((params, Some(data)))
    }

    fn create(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Resource> {
        info!("[TextureLoader] creates {:?}.", handle);

        item.0.validate(item.1.as_ref())?;

        let cmd = Command::CreateTexture(handle, item.0, item.1);
        self.frames.write().cmds.push(cmd);

        Ok(item.0)
    }

    fn delete(&self, handle: Self::Handle, _: Self::Resource) {
        info!("[TextureLoader] deletes {:?}.", handle);

        let cmd = Command::DeleteTexture(handle);
        self.frames.write().cmds.push(cmd);
    }
}
