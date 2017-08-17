use std::collections::HashMap;

use bincode;
use uuid;

use super::TextureItem;
use super::errors::*;

/// A atlas frame.
pub struct AtlasFrame {
    pub texture: TextureItem,
    pub size: (u16, u16),
    pub position: (u16, u16),
    pub pivot: (f32, f32),
}

/// Internal atlas frame.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AtlasInternalFrame {
    pub size: (u16, u16),
    pub position: (u16, u16),
    pub pivot: (f32, f32),
}

/// When designing sprite graphics, it is convenient to work with a separate
/// texture file for each character. However, a significant portion of a sprite
/// texture will often be taken up by the empty space between the graphic
/// elements and this space will result in wasted video memory at runtime.
///
/// For optimal performance, it is best to pack graphics from several sprite
/// textures tightly together within a single texture known as an `Atlas`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Atlas {
    /// The uuid of underlying texture.
    texture: uuid::Uuid,
    /// The scale factor of atlas frames.
    scale: f32,
    /// Atlas frames.
    frames: HashMap<String, AtlasInternalFrame>,
}

impl Atlas {
    pub fn new(texture: uuid::Uuid,
               scale: f32,
               frames: HashMap<String, AtlasInternalFrame>)
               -> Atlas {
        Atlas {
            texture: texture,
            scale: scale,
            frames: frames,
        }
    }

    /// Returns the uuid of underlying texture.
    #[inline]
    pub fn texture(&self) -> uuid::Uuid {
        self.texture
    }

    /// Returns the scale factor of atlas frames.
    #[inline]
    pub fn scale(&self) -> f32 {
        self.scale
    }
}

impl Atlas {
    /// Get a atlas frame with filename.
    #[inline]
    pub fn frame(&self, mut rs: &mut super::ResourceSystem, filename: &str) -> Option<AtlasFrame> {
        if let Some(frame) = self.frames.get(filename).and_then(|v| Some(v.clone())) {
            if let Ok(texture) = rs.load_texture_with_uuid(self.texture) {
                return Some(AtlasFrame {
                                texture: texture,
                                position: frame.position,
                                size: frame.size,
                                pivot: frame.pivot,
                            });
            }
        }

        None
    }
}

impl super::Resource for Atlas {
    fn size(&self) -> usize {
        0
    }
}

pub type AtlasSerializationPayload = Atlas;

impl super::ResourceLoader for Atlas {
    type Item = Atlas;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        let data = bincode::deserialize(&bytes)?;
        Ok(data)
    }
}