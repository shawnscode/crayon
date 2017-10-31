use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;

use resource;
use graphics;
use super::GraphicsResourceSystem;

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

impl resource::Resource for Texture {
    fn size(&self) -> usize {
        self.buf.len()
    }
}

impl resource::ResourceParser for Texture {
    type Item = Texture;

    fn parse(bytes: &[u8]) -> resource::errors::Result<Self> {
        use image;
        use image::GenericImage;

        let dynamic = image::load_from_memory(&bytes).unwrap().flipv();
        Ok(Texture::new(graphics::TextureFormat::U8U8U8U8,
                        dynamic.dimensions(),
                        dynamic.to_rgba().into_raw()))
    }
}

impl resource::ExternalResourceSystem for GraphicsResourceSystem<graphics::TextureHandle> {
    type Item = graphics::TextureHandle;
    type Data = Texture;
    type Options = graphics::TextureSetup;

    fn load(&mut self,
            path: &Path,
            src: &Self::Data,
            mut options: Self::Options)
            -> resource::errors::Result<Arc<Self::Item>> {
        let hash = path.into();
        if let Some(v) = self.arena.get(&hash) {
            return Ok(v.clone());
        }

        options.dimensions = src.dimensions();
        options.format = src.format();

        let handle = self.video
            .create_texture(options, src.data().to_owned())
            .unwrap();
        let handle = Arc::new(handle);
        self.arena.insert(hash, handle.clone());

        Ok(handle)
    }

    fn unload_unused(&mut self) {
        let mut next = HashMap::new();
        for (k, v) in self.arena.drain() {
            if Arc::strong_count(&v) > 1 {
                next.insert(k, v);
            } else {
                self.video.delete_texture(*v);
            }
        }
        self.arena = next;
    }
}