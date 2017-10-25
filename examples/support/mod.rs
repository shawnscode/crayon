use crayon::prelude::*;

pub struct TextureParser {}

impl resource::ResourceParser for TextureParser {
    type Item = Texture;

    fn parse(bytes: &[u8]) -> resource::errors::Result<Self::Item> {
        use image;
        use image::GenericImage;

        let dynamic = image::load_from_memory(&bytes).unwrap().flipv();
        Ok(Texture::new(graphics::TextureFormat::U8U8U8U8,
                        dynamic.dimensions(),
                        dynamic.to_rgba().into_raw()))
    }
}