pub mod manifest;
pub mod bytes;
pub mod texture;
pub mod atlas;
pub mod shader;
pub mod material;

pub use self::manifest::{ResourceManifest, ResourceManifestItem};
pub use self::bytes::BytesSerializationPayload;
pub use self::texture::TextureSerializationPayload;
pub use self::atlas::AtlasSerializationPayload;
pub use self::shader::ShaderSerializationPayload;
pub use self::material::MaterialSerializationPayload;

/// Implements `ResourceSerializationLoader` to indicate how we load a serialized
/// resource which has metadata included.
pub trait BuildinResource
    : super::Resource + super::ResourceIndex + Sized + 'static {
    type Loader: super::ResourceLoader<Item = Self>;

    /// Get the underlying payload type of this loader.
    fn payload() -> ResourceType;
}

macro_rules! declare_buildin_resource {
    ($($payload: ident => $name: ident;)*) => (
        /// Payload type of the underlying serialization data.
        #[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
        pub enum ResourceType {
            $($name,)*
        }

        /// Register all the resource type which has build-in supports with `crayon-workflow`.
        pub fn register(frontend: &mut super::ResourceFrontend) {
            $(frontend.register::<super::$name>();)*
        }
        
        $(impl BuildinResource for super::$name {
            type Loader = $payload;

            fn payload() -> ResourceType {
                ResourceType::$name
            }
        })*
    )
}

declare_buildin_resource!{
    BytesSerializationPayload => Bytes;
    TextureSerializationPayload => Texture;
    AtlasSerializationPayload => Atlas;
    ShaderSerializationPayload => Shader;
    MaterialSerializationPayload => Material;
}