use std::sync::Arc;

use rusttype;
use crayon::{graphics, math};

pub struct Font {
    bytes: Vec<u8>,
    cache: rusttype::gpu_cache::Cache,
    texture: Option<graphics::TextureHandle>,
}

impl_handle!(FontHandle);

impl Font {
    /// Queues a glyph for caching.
    pub fn add(&mut self, glyph: rusttype::PositionedGlyph) {
        self.cache.queue_glyph(0, glyph);
    }

    /// Updates cached font data into video memory.
    pub fn update_video_texture(&mut self,
                                shared: &graphics::GraphicsSystemShared)
                                -> graphics::errors::Result<()> {
        let handle = if self.texture.is_none() {
            let mut setup = graphics::TextureSetup::default();
            setup.filter = graphics::TextureFilter::Linear;
            setup.mipmap = false;
            setup.dimensions = (512, 512);
            setup.format = graphics::TextureFormat::U8;

            shared.create_texture(setup, None)?
        } else {
            self.texture.unwrap()
        };

        self.cache
            .cache_queued(|rect, data| {
                              let rect = graphics::Rect::new(math::Point2::new(rect.min.x as i32,
                                                                               rect.min.y as i32),
                                                             math::Point2::new(rect.max.x as i32,
                                                                               rect.max.y as i32));
                              shared.update_texture(handle, rect, data);
                          })
            .unwrap();

        Ok(())
    }

    /// Returns the video texture handle.
    pub fn video_texture(&self) -> Option<graphics::TextureHandle> {
        self.texture
    }
}