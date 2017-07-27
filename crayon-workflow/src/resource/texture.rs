use crayon::graphics;

/// Compression settings of texture.
#[derive(Debug, Serialize, Deserialize)]
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
}