use image;
use image::GenericImage;
use graphics;

use super::errors::*;
use super::ResourceItem;

#[derive(Debug)]
pub struct Texture {
    video: Option<graphics::TextureHandle>,
    address: graphics::TextureAddress,
    filter: graphics::TextureFilter,
    mipmap: bool,
    dimensions: (u32, u32),
    buf: Vec<u8>,
}

impl ResourceItem for Texture {
    fn from_bytes(bytes: &[u8]) -> Result<Self>
        where Self: Sized
    {
        let dynamic = image::load_from_memory(bytes)?;
        Ok(Texture {
            video: None,
            address: graphics::TextureAddress::Clamp,
            filter: graphics::TextureFilter::Linear,
            mipmap: false,
            dimensions: dynamic.dimensions(),
            buf: dynamic.to_rgba().into_raw(),
        })
    }

    fn bytes(&self) -> usize {
        self.buf.len()
    }

    fn update_video_object(&mut self, video: &mut graphics::Graphics) -> Result<()> {
        if self.video.is_none() {
            let handle = video.create_texture(graphics::TextureFormat::U8U8U8U8,
                                self.address,
                                self.filter,
                                self.mipmap,
                                self.dimensions.0,
                                self.dimensions.1,
                                self.buf.as_slice())?;
            self.video = Some(handle);
        }

        Ok(())
    }

    fn delete_video_object(&mut self, video: &mut graphics::Graphics) -> Result<()> {
        if let Some(handle) = self.video {
            video.delete_texture(handle)?;
        }

        Ok(())
    }
}

impl Texture {
    pub fn video_object(&self) -> Option<graphics::TextureHandle> {
        self.video
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.dimensions.1
    }
}
