pub mod metadata;
pub mod database;

pub mod texture;
pub mod bytes;
pub mod atlas;
pub mod shader;
pub mod material;

pub use self::database::ResourceDatabase;
pub use self::metadata::ResourceMetadata;

pub use self::texture::TextureMetadata;
pub use self::bytes::BytesMetadata;
pub use self::atlas::AtlasMetadata;
pub use self::shader::ShaderMetadata;
pub use self::material::MaterialMetadata;

pub use crayon::resource::workflow::BuildinResourceType;
/// The enumeration of all the fundamental resources that could be imported into
/// workspace.
pub use crayon::resource::workflow::BuildinResourceType as Resource;

const METADATA_EXTENSION: &'static str = "meta";