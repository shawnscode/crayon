use std;
use std::path::PathBuf;
use std::collections::HashMap;

use crayon::graphics;
use crayon::utils::HashValue;

/// A atlas frame.
#[derive(Debug)]
pub struct AtlasFrame {
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
    /// The path of underlaying texture.
    pub texture: PathBuf,
    /// The diemensions of texture.
    pub dimensions: (i32, i32),
    /// The scale factor of atlas frames.
    pub scale: f32,
    /// Atlas frames.
    pub frames: HashMap<HashValue<str>, AtlasInternalFrame>,
}


pub trait AtlasParser {
    type Error: std::error::Error + std::fmt::Debug;

    fn parse(bytes: &[u8]) -> std::result::Result<Atlas, Self::Error>;
}

impl Atlas {
    /// Get a atlas frame with filename.
    #[inline]
    pub fn frame<S>(&self, filename: S) -> Option<AtlasFrame>
        where S: AsRef<str>
    {
        let filename = filename.as_ref().into();
        if let Some(frame) = self.frames.get(&filename).and_then(|v| Some(v.clone())) {
            let (w, h) = {
                (self.dimensions.0 as f32 * self.scale, self.dimensions.1 as f32 * self.scale)
            };

            let nposition = (1f32 - frame.position.0 as f32 / w,
                             1f32 - frame.position.1 as f32 / h);
            let nsize = (frame.size.0 as f32 / w, frame.size.1 as f32 / h);

            return Some(AtlasFrame {
                            position: nposition,
                            size: nsize,
                            pivot: frame.pivot,
                        });
        }

        None
    }
}