use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;

use uuid;

use errors::*;
use super::*;

macro_rules! concrete_metadata_decl {
    ($($name: ident => $metadata: ident,)*) => (
        #[derive(Debug, Serialize, Deserialize)]
        pub enum ResourceConcreteMetadata {
            $($name($metadata),)*
        }
        
        impl ResourceConcreteMetadata {
            pub fn new(tt: Resource) -> Self {
                match tt {
                    $(ResourcePayload::$name => ResourceConcreteMetadata::$name($metadata::new()),)*
                }
            }

            pub fn payload(&self) -> Resource {
                match self {
                    $(&ResourceConcreteMetadata::$name(_) => ResourcePayload::$name,)*
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
}

pub trait ResourceUnderlyingMetadata {
    /// Check the validation of resource.
    fn validate(&self, bytes: &[u8]) -> Result<()>;

    /// Build the resource into runtime serialization data.
    fn build(&self,
             database: &ResourceDatabase,
             path: &Path,
             bytes: &[u8],
             out: &mut Vec<u8>)
             -> Result<()>;
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
    pub fn is(&self, tt: ResourcePayload) -> bool {
        self.metadata.payload() == tt
    }

    #[inline]
    pub fn payload(&self) -> ResourcePayload {
        self.metadata.payload()
    }

    #[inline]
    pub fn validate(&self, bytes: &[u8]) -> Result<()> {
        self.metadata.underlying().validate(&bytes)
    }

    #[inline]
    pub fn build(&self,
                 database: &database::ResourceDatabase,
                 path: &Path,
                 bytes: &[u8],
                 mut out: &mut Vec<u8>)
                 -> Result<()> {
        self.metadata
            .underlying()
            .build(&database, &path, &bytes, &mut out)
    }
}