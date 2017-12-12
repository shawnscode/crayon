use crayon::prelude::*;

use image;
use image::GenericImage;

use std::time::Duration;

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

pub fn to_ms(duration: Duration) -> f32 {
    duration.as_secs() as f32 * 1000.0 + duration.subsec_nanos() as f32 / 1_000_000.0
}