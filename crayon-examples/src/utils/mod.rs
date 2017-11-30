use crayon::prelude::*;

pub struct TextureParser {}

impl assets::texture_sys::TextureFormat for TextureParser {
    fn parse(bytes: &[u8]) -> assets::texture_sys::Result<Texture> {
        use image;
        use image::GenericImage;

        let dynamic = image::load_from_memory(&bytes).unwrap().flipv();
        Ok(Texture::new(graphics::TextureFormat::U8U8U8U8,
                        dynamic.dimensions(),
                        dynamic.to_rgba().into_raw()))
    }
}