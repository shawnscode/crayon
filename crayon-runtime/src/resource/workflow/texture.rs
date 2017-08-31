use image;
use image::GenericImage;
use bincode;

use graphics;
use super::super::errors::*;
use super::super::{ResourceLoader, ResourceFrontend, texture};

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureSerializationPayload {
    pub mipmap: bool,
    pub address: graphics::TextureAddress,
    pub filter: graphics::TextureFilter,
    pub is_compressed: bool,
    pub bytes: Vec<u8>,
}

impl ResourceLoader for TextureSerializationPayload {
    type Item = texture::Texture;

    fn load_from_memory(_: &mut ResourceFrontend, bytes: &[u8]) -> Result<Self::Item> {
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