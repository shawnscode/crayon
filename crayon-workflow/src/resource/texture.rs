use crayon::graphics;
use image;

use errors::*;

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
    pub fn new() -> TextureMetadata {
        TextureMetadata {
            mipmap: false,
            address: graphics::TextureAddress::Clamp,
            filter: graphics::TextureFilter::Linear,
            compression: TextureCompressionMetadata::None,
        }
    }

    pub fn validate(&self, bytes: &[u8]) -> Result<()> {
        image::guess_format(&bytes)?;
        Ok(())
    }

    pub fn build(&self, bytes: &[u8], mut out: &mut Vec<u8>) -> Result<()> {
        assert!(self.compression == TextureCompressionMetadata::None);
        let src = image::load_from_memory(&bytes)?;
        src.save(&mut out, image::ImageFormat::WEBP)?;
        Ok(())
    }
}