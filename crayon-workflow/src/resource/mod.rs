pub mod metadata;
pub mod database;

pub mod texture;
pub mod bytes;
pub mod atlas;

pub use self::database::ResourceDatabase;
pub use self::metadata::ResourceMetadata;

pub use self::texture::TextureMetadata;
pub use self::bytes::BytesMetadata;
pub use self::atlas::AtlasMetadata;

/// The enumeration of all the fundamental resources that could be imported into
/// workspace.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Resource {
    Bytes,
    Texture,
    Atlas,
}

const METADATA_EXTENSION: &'static str = "meta";

use crayon;

impl Into<crayon::resource::manifest::ResourcePayload> for Resource {
    fn into(self) -> crayon::resource::manifest::ResourcePayload {
        match self {
            Resource::Bytes => crayon::resource::manifest::ResourcePayload::Bytes,
            Resource::Texture => crayon::resource::manifest::ResourcePayload::Texture,
            Resource::Atlas => crayon::resource::manifest::ResourcePayload::Atlas,
        }
    }
}