pub mod manifest;
pub mod bytes;
pub mod texture;
pub mod atlas;
pub mod shader;

pub use self::manifest::{ResourceManifest, ResourceManifestItem};
pub use self::bytes::BytesSerializationPayload;
pub use self::texture::TextureSerializationPayload;
pub use self::atlas::AtlasSerializationPayload;
pub use self::shader::ShaderSerializationPayload;

/// Payload type of the underlying serialization data.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum ResourcePayload {
    Bytes,
    Texture,
    Atlas,
    Shader,
}

/// Implements `ResourceSerializationLoader` to indicate how we load a serialized
/// resource which has metadata included.
pub trait ResourceSerialization
    : super::Resource + super::ResourceIndex + Sized + 'static {
    type Loader: super::ResourceLoader<Item = Self>;

    /// Get the underlying payload type of this loader.
    fn payload() -> ResourcePayload;
}

/// Register all the resource type which has build-in supports with `crayon-workflow`.
pub fn register(frontend: &mut super::ResourceSystem) {
    frontend.register::<super::Bytes>();
    frontend.register::<super::Texture>();
    frontend.register::<super::Atlas>();
    frontend.register::<super::Shader>();
}