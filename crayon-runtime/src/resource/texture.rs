use graphics;

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
    pub fn new(dimensions: (u32, u32),
               buf: Vec<u8>,
               mipmap: bool,
               address: graphics::TextureAddress,
               filter: graphics::TextureFilter)
               -> Texture {
        Texture {
            mipmap: mipmap,
            address: address,
            filter: filter,
            dimensions: dimensions,
            buf: buf,
            video: None,
        }
    }

    pub fn update_video_object(&mut self,
                               video: &mut graphics::Graphics)
                               -> graphics::errors::Result<()> {
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