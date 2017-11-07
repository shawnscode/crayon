use graphics;
use resource;

///
#[derive(Debug, Clone)]
pub struct Texture {
    format: graphics::TextureFormat,
    dimensions: (u32, u32),
    buf: Vec<u8>,
}

impl Texture {
    pub fn new(format: graphics::TextureFormat, dimensions: (u32, u32), buf: Vec<u8>) -> Self {
        Texture {
            format: format,
            dimensions: dimensions,
            buf: buf,
        }
    }

    #[inline]
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.buf
    }

    #[inline]
    pub fn format(&self) -> graphics::TextureFormat {
        self.format
    }
}

impl resource::cache::Meansurable for Texture {
    fn size(&self) -> usize {
        self.buf.len()
    }
}