use crayon::prelude::*;

pub struct TextureParser {}

impl graphics::TextureParser for TextureParser {
    fn parse(bytes: &[u8]) -> graphics::errors::Result<graphics::Texture> {
        use image;
        use image::GenericImage;

        let dynamic = image::load_from_memory(&bytes).unwrap().flipv();
        Ok(graphics::Texture {
               format: graphics::TextureFormat::U8U8U8U8,
               dimensions: dynamic.dimensions(),
               data: dynamic.to_rgba().into_raw(),
           })
    }
}