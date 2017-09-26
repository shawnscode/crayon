//! Manipulations of resources in workspace.

pub mod texture;
pub mod bytes;
pub mod atlas;
pub mod shader;
pub mod material;

pub use self::texture::TextureDesc;
pub use self::bytes::BytesDesc;
pub use self::atlas::TexturePackerAtlasDesc;
pub use self::shader::ShaderDesc;
pub use self::material::MaterialDesc;

pub use crayon::resource::workflow::ResourceType;

use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;

use uuid;
use errors::*;
use workspace::Database;

macro_rules! metadata_decl {
    ($($metadata: ident => ($name: ident, $format: ident),)*) => (
        #[derive(Debug, Serialize, Deserialize)]
        pub enum ResourceMetadataDesc {
            $($name($metadata),)*
        }

        impl ResourceMetadataDesc {
            pub fn format(&self) -> ResourceType {
                match self {
                    $(&ResourceMetadataDesc::$name(_) => ResourceType::$format,)*
                }
            }

            pub fn as_builder(&self) -> &ResourceMetadataHandler {
                match self {
                    $(&ResourceMetadataDesc::$name(ref mt) => mt,)*
                }
            }
        }

        $(
        impl Into<ResourceMetadataDesc> for $metadata {
            fn into(self) -> ResourceMetadataDesc {
                ResourceMetadataDesc::$name(self)
            }
        }
        )*
    )
}

metadata_decl! {
    BytesDesc => (Bytes, Bytes),
    TextureDesc => (Texture, Texture),
    TexturePackerAtlasDesc => (TexturePackerAtlas, Atlas),
}

pub trait ResourceMetadataHandler {
    /// Check the validation of resource.
    fn validate(&self, _: &[u8]) -> Result<()>;

    /// Build the resource into runtime serialization data.
    fn build(&self, _: &Database, _: &Path, _: &[u8], _: &mut Vec<u8>) -> Result<()>;
}

/// The descriptions of a resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMetadata {
    time_created: u64,
    uuid: uuid::Uuid,
    desc: ResourceMetadataDesc,
}

impl Default for ResourceMetadata {
    fn default() -> Self {
        ResourceMetadata::new(ResourceMetadataDesc::Bytes(BytesDesc::new()))
    }
}

impl ResourceMetadata {
    pub fn new(desc: ResourceMetadataDesc) -> ResourceMetadata {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ResourceMetadata {
            time_created: timestamp,
            uuid: uuid::Uuid::new_v4(),
            desc: desc,
        }
    }

    pub fn new_with_default(tt: ResourceType) -> Option<ResourceMetadata> {
        let desc = match tt {
            ResourceType::Bytes => ResourceMetadataDesc::Bytes(BytesDesc::new()),
            ResourceType::Texture => ResourceMetadataDesc::Texture(TextureDesc::new()),
            _ => return None,
        };

        Some(ResourceMetadata::new(desc))
    }

    #[inline]
    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    #[inline]
    pub fn is(&self, tt: ResourceType) -> bool {
        self.desc.format() == tt
    }

    #[inline]
    pub fn format(&self) -> ResourceType {
        self.desc.format()
    }

    #[inline]
    pub fn validate(&self, bytes: &[u8]) -> Result<()> {
        self.desc.as_builder().validate(&bytes)
    }

    #[inline]
    pub fn build(&self,
                 database: &Database,
                 path: &Path,
                 bytes: &[u8],
                 mut out: &mut Vec<u8>)
                 -> Result<()> {
        self.desc
            .as_builder()
            .build(&database, &path, &bytes, &mut out)
    }
}