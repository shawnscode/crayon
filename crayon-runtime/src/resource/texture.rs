use image;
use image::GenericImage;
use bincode;
use graphics;

use super::errors::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureSerializationPayload {
    pub mipmap: bool,
    pub address: graphics::TextureAddress,
    pub filter: graphics::TextureFilter,
    pub is_compressed: bool,
    pub bytes: Vec<u8>,
}

impl super::ResourceLoader for TextureSerializationPayload {
    type Item = Texture;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        let data: TextureSerializationPayload = bincode::deserialize(&bytes)?;
        assert!(!data.is_compressed);

        let dynamic = image::load_from_memory(&data.bytes)?;

        Ok(Texture {
               mipmap: data.mipmap,
               address: data.address,
               filter: data.filter,
               dimensions: dynamic.dimensions(),
               buf: dynamic.to_rgba().into_raw(),
               video: None,
           })
    }
}

#[derive(Debug)]
pub struct Texture {
    mipmap: bool,
    address: graphics::TextureAddress,
    filter: graphics::TextureFilter,
    dimensions: (u32, u32),
    buf: Vec<u8>,
    video: Option<graphics::TextureRef>,
}

impl Texture {
    pub fn update_video_object(&mut self, video: &mut graphics::Graphics) -> Result<()> {
        if self.video.is_none() {
            let v = video
                .create_texture(graphics::TextureFormat::U8U8U8U8,
                                self.address,
                                self.filter,
                                self.mipmap,
                                self.dimensions.0,
                                self.dimensions.1,
                                self.buf.as_slice())?;
            self.video = Some(v);
        }

        Ok(())
    }

    pub fn update_video_parameters(&mut self,
                                   address: graphics::TextureAddress,
                                   filter: graphics::TextureFilter) {
        self.address = address;
        self.filter = filter;

        if let Some(ref mut v) = self.video {
            v.object.write().unwrap().update_parameters(address, filter);
        }
    }

    pub fn video_object(&self) -> Option<graphics::TextureHandle> {
        self.video.as_ref().map(|v| v.handle)
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

impl super::Resource for Texture {
    fn size(&self) -> usize {
        self.buf.len()
    }
}

impl super::ResourceLoader for Texture {
    type Item = Texture;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        TextureSerializationPayload::load_from_memory(&bytes)
    }
}