pub mod errors;
pub mod archive;
pub mod cache;
pub mod backend;
pub mod manifest;
pub mod system;

pub mod texture;
pub mod bytes;
pub mod atlas;

pub use self::errors::*;
pub use self::archive::{File, Archive, FilesystemArchive, ZipArchive, ArchiveCollection};
pub use self::cache::Cache;
pub use self::backend::{ResourceSystemBackend, ResourceLoader};
pub use self::system::ResourceSystem;

pub use self::texture::Texture;
pub use self::bytes::Bytes;
pub use self::atlas::Atlas;

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};

/// `Resource`.
pub trait Resource {
    /// Returns internal memory usages of resource in bytes.
    fn size(&self) -> usize;
}

/// `ResourceIndex`.
pub trait ResourceIndex {
    fn type_index() -> usize;
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