use bincode;
use std::io::Cursor;
use std::sync::Arc;

use errors::*;

use super::super::backends::frame::Command;
use super::super::DoubleFrame;
use super::texture::*;

pub const MAGIC: [u8; 8] = [
    'V' as u8, 'T' as u8, 'E' as u8, 'X' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Clone)]
pub struct TextureLoader {
    frames: Arc<DoubleFrame>,
}

impl TextureLoader {
    pub(crate) fn new(frames: Arc<DoubleFrame>) -> Self {
        TextureLoader { frames: frames }
    }
}

impl ::res::registry::Register for TextureLoader {
    type Handle = TextureHandle;
    type Intermediate = (TextureParams, Option<TextureData>);
    type Value = TextureParams;

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

    fn attach(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Value> {
        item.0.validate(item.1.as_ref())?;

        let mut frame = self.frames.front();
        let task = Command::CreateTexture(handle, item.0, item.1);
        frame.cmds.push(task);

        Ok(item.0)
    }

    fn detach(&self, handle: Self::Handle, _: Self::Value) {
        let cmd = Command::DeleteTexture(handle);
        self.frames.front().cmds.push(cmd);
    }
}
