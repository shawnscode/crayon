pub mod errors;
pub mod archive;
pub mod cache;
pub mod backend;
pub mod system;
pub mod workflow;

pub mod texture;
pub mod bytes;
pub mod atlas;

pub use self::errors::*;
pub use self::archive::{File, Archive, FilesystemArchive, ZipArchive, ArchiveCollection};
pub use self::cache::Cache;
pub use self::backend::ResourceSystemBackend;
pub use self::system::ResourceSystem;

pub use self::texture::Texture;
pub use self::bytes::Bytes;
pub use self::atlas::{Atlas, AtlasFrame};

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;

/// `Resource`.
pub trait Resource {
    /// Returns internal memory usages of resource in bytes.
    fn size(&self) -> usize;
}

/// `ResourceIndex`.
pub trait ResourceIndex {
    fn type_index() -> usize;
}

/// This trait addresses how we load a specified resource `ResourceLoader::Item`
/// into runtime.
pub trait ResourceLoader: Debug {
    type Item: Resource + ResourceIndex + 'static;

    /// Load resource from a file on disk.
    fn load_from_file(file: &mut archive::File) -> Result<Self::Item> {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Self::load_from_memory(&buf)
    }

    /// Create resource from memory region.
    fn load_from_memory(bytes: &[u8]) -> Result<Self::Item>;
}

lazy_static! {
    /// Lazy initialized id of resource. Which produces a continuous index address.
    #[doc(hidden)]
    pub static ref _INDEX: AtomicUsize = AtomicUsize::new(0);
}

#[macro_export]
macro_rules! declare_resource {
    ( $ITEM:ident ) => {
        impl $crate::resource::ResourceIndex for $ITEM {
            fn type_index() -> usize {
                use std::sync::atomic::Ordering;
                use $crate::resource::_INDEX;
                lazy_static!{static ref ID: usize = _INDEX.fetch_add(1, Ordering::SeqCst);};
                *ID
            }
        }
    };
}

pub type TextureItem = Arc<RwLock<Texture>>;
declare_resource!(Texture);

pub type BytesItem = Arc<RwLock<Bytes>>;
declare_resource!(Bytes);

pub type AtlasItem = Arc<RwLock<Atlas>>;
declare_resource!(Atlas);