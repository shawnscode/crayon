use crayon::prelude::*;

use image;
use image::GenericImage;

pub struct TextureParser {}

impl graphics::TextureParser for TextureParser {
    type Error = image::ImageError;

    fn parse(bytes: &[u8]) -> image::ImageResult<graphics::TextureData> {
        let dynamic = image::load_from_memory(&bytes)?.flipv();
        Ok(graphics::TextureData {
               format: graphics::TextureFormat::U8U8U8U8,
               dimensions: dynamic.dimensions(),
               data: dynamic.to_rgba().into_raw(),
           })
    }
}