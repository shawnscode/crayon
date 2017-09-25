use crayon::graphics;
use crayon::resource;
use image;
use bincode;

use std::path::Path;
use errors::*;
use workspace::Database;
use super::ResourceUnderlyingMetadata;

/// Compression settings of texture.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextureCompressionMetadata {
    /// Texture will not be compressed.
    None,
    /// Texture will be compressed using a standard format depending on the platform (DXT, ASTC, ...).
    Compressed,
}

/// `TextureMetadata` contains settings that used to address how should we process
/// and import corresponding texture resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct TextureMetadata {
    /// Generate mip maps for the texture?
    pub mipmap: bool,
    /// Wrap mode (usually repeat or clamp) of the texture.
    pub address: graphics::TextureAddress,
    /// Filtering mode of the texture.
    pub filter: graphics::TextureFilter,
    /// Compression of imported texture.
    pub compression: TextureCompressionMetadata,
}

impl TextureMetadata {
    pub fn new() -> Self {
        TextureMetadata {
            mipmap: false,
            address: graphics::TextureAddress::Clamp,
            filter: graphics::TextureFilter::Linear,
            compression: TextureCompressionMetadata::None,
        }
    }
}

impl ResourceUnderlyingMetadata for TextureMetadata {
    fn validate(&self, bytes: &[u8]) -> Result<()> {
        image::guess_format(&bytes)?;
        Ok(())
    }

    fn build(&self, _: &Database, _: &Path, data: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        assert!(self.compression == TextureCompressionMetadata::None);

        let mut bytes = Vec::new();
        let src = image::load_from_memory(&data)?;
        src.save(&mut bytes, image::ImageFormat::PNG)?;

        let payload = resource::workflow::TextureSerializationPayload {
            mipmap: self.mipmap,
            address: self.address,
            filter: self.filter,
            is_compressed: false,
            bytes: bytes,
        };

        bincode::serialize_into(&mut out, &payload, bincode::Infinite)?;
        Ok(())
    }
}