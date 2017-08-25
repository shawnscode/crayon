pub mod metadata;
pub mod database;

pub mod texture;
pub mod bytes;
pub mod atlas;
pub mod shader;

pub use self::database::ResourceDatabase;
pub use self::metadata::ResourceMetadata;

pub use self::texture::TextureMetadata;
pub use self::bytes::BytesMetadata;
pub use self::atlas::AtlasMetadata;
pub use self::shader::ShaderMetadata;

use crayon;

/// The enumeration of all the fundamental resources that could be imported into
/// workspace.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Resource {
    Bytes,
    Texture,
    Atlas,
    Shader,
}

const METADATA_EXTENSION: &'static str = "meta";

impl Into<crayon::resource::workflow::ResourcePayload> for Resource {
    fn into(self) -> crayon::resource::workflow::ResourcePayload {
        match self {
            Resource::Bytes => crayon::resource::workflow::ResourcePayload::Bytes,
            Resource::Texture => crayon::resource::workflow::ResourcePayload::Texture,
            Resource::Atlas => crayon::resource::workflow::ResourcePayload::Atlas,
            Resource::Shader => crayon::resource::workflow::ResourcePayload::Shader,
        }
    }
}