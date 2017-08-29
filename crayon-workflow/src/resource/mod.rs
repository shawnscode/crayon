//! # Resource
//!
//! Its hard to make a decent game, especially when you are dealing with massive resources,
//! scripts etc. Its always trivial and error-prone to make a plain file produced by some kind
//! of authoring tools into runtime resource. There are some common senarios when
//!
//! - The assets might be modified by artiest continuous, so it would be great if we store resource
//! in formats which could producing and editing by authoring tools directly.
//! - The most effecient format is dependent on platform and hardware devices. The assets might be
//! converts to various formats based on the build target before packing into playable package.
//! - The processing of assets from plain formats into runtime formats might causes heavily cpu consumption,
//! and takes minutes for medium size project. By the same time, its a common requirement to edit
//! and preview the effects on playable environment. So we should have some kind of mechanism to manage
//! the asset processing incrementally.
//!
//! To accomplish this goal, we does four tools.
//! - Imports resources and manage various kind of additional data about them for you.
//! - Converts plain resources produced by authoring tools into crayon supported format.

pub mod texture;
pub mod bytes;
pub mod atlas;
pub mod shader;
pub mod material;

pub use self::texture::TextureMetadata;
pub use self::bytes::BytesMetadata;
pub use self::atlas::AtlasMetadata;
pub use self::shader::ShaderMetadata;
pub use self::material::MaterialMetadata;

/// The enumeration of all the fundamental resources that could be imported into
/// workspace.
pub use crayon::resource::workflow::BuildinResourceType;
pub use crayon::resource::workflow::BuildinResourceType as Resource;

use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;

use uuid;
use errors::*;
use workspace::Database;

macro_rules! concrete_metadata_decl {
    ($($name: ident => $metadata: ident,)*) => (
        #[derive(Debug, Serialize, Deserialize)]
        pub enum ResourceConcreteMetadata {
            $($name($metadata),)*
        }
        
        impl ResourceConcreteMetadata {
            pub fn new(tt: Resource) -> Self {
                match tt {
                    $(BuildinResourceType::$name => ResourceConcreteMetadata::$name($metadata::new()),)*
                }
            }

            pub fn payload(&self) -> Resource {
                match self {
                    $(&ResourceConcreteMetadata::$name(_) => BuildinResourceType::$name,)*
                }
            }

            pub fn is(&self, tt: Resource) -> bool {
                self.payload() == tt
            }

            pub fn underlying(&self) -> &ResourceUnderlyingMetadata {
                match self {
                    $(&ResourceConcreteMetadata::$name(ref mt) => mt,)*
                }
            }
        }
    )
}

concrete_metadata_decl! {
    Bytes => BytesMetadata,
    Texture => TextureMetadata,
    Atlas => AtlasMetadata,
    Shader => ShaderMetadata,
    Material => MaterialMetadata,
}

pub trait ResourceUnderlyingMetadata {
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
    metadata: ResourceConcreteMetadata,
}

impl ResourceMetadata {
    pub fn new(metadata: ResourceConcreteMetadata) -> ResourceMetadata {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ResourceMetadata {
            time_created: timestamp,
            uuid: uuid::Uuid::new_v4(),
            metadata: metadata,
        }
    }

    pub fn new_as(tt: Resource) -> ResourceMetadata {
        ResourceMetadata::new(ResourceConcreteMetadata::new(tt))
    }

    #[inline]
    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    #[inline]
    pub fn is(&self, tt: BuildinResourceType) -> bool {
        self.metadata.payload() == tt
    }

    #[inline]
    pub fn payload(&self) -> BuildinResourceType {
        self.metadata.payload()
    }

    #[inline]
    pub fn validate(&self, bytes: &[u8]) -> Result<()> {
        self.metadata.underlying().validate(&bytes)
    }

    #[inline]
    pub fn build(&self,
                 database: &Database,
                 path: &Path,
                 bytes: &[u8],
                 mut out: &mut Vec<u8>)
                 -> Result<()> {
        self.metadata
            .underlying()
            .build(&database, &path, &bytes, &mut out)
    }
}