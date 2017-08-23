use image;
use image::GenericImage;

use bincode;

use graphics;
use super::super::errors::*;
use super::super::texture;

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureSerializationPayload {
    pub mipmap: bool,
    pub address: graphics::TextureAddress,
    pub filter: graphics::TextureFilter,
    pub is_compressed: bool,
    pub bytes: Vec<u8>,
}

impl super::super::ResourceLoader for TextureSerializationPayload {
    type Item = texture::Texture;

    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item> {
        let data: TextureSerializationPayload = bincode::deserialize(&bytes)?;
        assert!(!data.is_compressed);

        let dynamic = image::load_from_memory(&data.bytes)?.flipv();
        Ok(texture::Texture::new(dynamic.dimensions(),
                                 dynamic.to_rgba().into_raw(),
                                 data.mipmap,
                                 data.address,
                                 data.filter))
    }
}

impl super::ResourceSerialization for texture::Texture {
    type Loader = TextureSerializationPayload;

    fn payload() -> super::ResourcePayload {
        super::ResourcePayload::Texture
    }
}