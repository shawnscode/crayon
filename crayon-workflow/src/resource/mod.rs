pub mod texture;
pub mod metadata;
pub mod database;

pub use self::database::ResourceDatabase;
pub use self::metadata::ResourceMetadata;
pub use self::texture::TextureMetadata;

/// The enumeration of all the fundamental resources that could be imported into
/// workspace.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Resource {
    Bytes,
    Texture,
}

const METADATA_EXTENSION: &'static str = "meta";