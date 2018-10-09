use bincode;
use std::io::Cursor;
use std::sync::Arc;

use errors::*;
use res::prelude::*;
use utils::FastHashMap;

use super::super::VideoSystemShared;
use super::atlas::*;

pub const MAGIC: [u8; 8] = [
    'V' as u8, 'A' as u8, 'T' as u8, 'L' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Clone)]
pub struct AtlasLoader {
    res: Arc<ResourceSystemShared>,
    video: Arc<VideoSystemShared>,
}

impl AtlasLoader {
    pub(crate) fn new(res: Arc<ResourceSystemShared>, video: Arc<VideoSystemShared>) -> Self {
        AtlasLoader {
            res: res,
            video: video,
        }
    }
}

impl Register for AtlasLoader {
    type Handle = AtlasHandle;
    type Intermediate = Atlas;
    type Value = Atlas;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        if &bytes[0..8] != &MAGIC[..] {
            bail!("[AtlasLoader] MAGIC number not match.");
        }

        let mut file = Cursor::new(&bytes[8..]);
        let serializable: SerializableAtlas = bincode::deserialize_from(&mut file)?;
        let texture = self.video.create_texture_from_uuid(serializable.texture)?;

        self.res.wait_until(serializable.texture)?;

        info!(
            "[AtlasLoader] loads {:?}. (Sprites: {}, Vertices: {})",
            handle,
            serializable.sprites.len(),
            serializable.vertices.len(),
        );

        let mut sprites = FastHashMap::default();
        for v in serializable.sprites {
            let vertices = Vec::from(&serializable.vertices[v.start..v.end]);
            sprites.insert(v.name.into(), Arc::new(vertices));
        }

        Ok(Atlas {
            texture: texture,
            sprites: sprites,
        })
    }

    fn attach(&self, _: Self::Handle, item: Self::Intermediate) -> Result<Self::Value> {
        Ok(item)
    }

    fn detach(&self, _: Self::Handle, value: Self::Value) {
        self.video.delete_texture(value.texture);
    }
}
