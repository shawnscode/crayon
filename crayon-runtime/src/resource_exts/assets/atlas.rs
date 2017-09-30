use std::collections::HashMap;

use super::TexturePtr;

/// A atlas frame.
#[derive(Debug)]
pub struct AtlasFrame {
    pub texture: TexturePtr,
    /// Normalized position of the underlying texture.
    pub position: (f32, f32),
    /// Normalized rect of the underlying texture.
    pub size: (f32, f32),
    /// Normalized location of the frame's center point.
    pub pivot: (f32, f32),
}

/// Internal atlas frame.
#[derive(Debug, Copy, Clone)]
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
#[derive(Debug)]
pub struct Atlas {
    /// The uuid of underlying texture.
    texture: TexturePtr,
    /// The scale factor of atlas frames.
    scale: f32,
    /// Atlas frames.
    frames: HashMap<String, AtlasInternalFrame>,
}

impl Atlas {
    pub fn new(texture: TexturePtr,
               scale: f32,
               frames: HashMap<String, AtlasInternalFrame>)
               -> Atlas {
        Atlas {
            texture: texture,
            scale: scale,
            frames: frames,
        }
    }

    /// Returns the underlying texture of this atlas.
    #[inline]
    pub fn texture(&self) -> &TexturePtr {
        &self.texture
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
    pub fn frame(&self, filename: &str) -> Option<AtlasFrame> {
        if let Some(frame) = self.frames.get(filename).and_then(|v| Some(v.clone())) {
            let texture = self.texture.clone();
            let (w, h) = {
                let texture = texture.read().unwrap();
                (texture.width() as f32 * self.scale, texture.height() as f32 * self.scale)
            };

            let nposition = (1f32 - frame.position.0 as f32 / w,
                             1f32 - frame.position.1 as f32 / h);
            let nsize = (frame.size.0 as f32 / w, frame.size.1 as f32 / h);

            return Some(AtlasFrame {
                            texture: texture,
                            position: nposition,
                            size: nsize,
                            pivot: frame.pivot,
                        });
        }

        None
    }
}

impl super::super::Resource for Atlas {
    fn size(&self) -> usize {
        0
    }
}