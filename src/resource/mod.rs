pub mod errors;
pub mod archive;
pub mod cache;
pub mod resource;
pub mod texture;

pub use self::archive::{Read, Seek, Archive, FilesystemArchive, ZipArchive, ArchiveCollection};
pub use self::cache::Cache;
pub use self::texture::Texture;
pub use self::resource::ResourceSystem;

use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;

lazy_static! {
    /// Lazy initialized id of component. Which produces a continuous index address.
    #[doc(hidden)]
    static ref _INDEX: AtomicUsize = AtomicUsize::new(0);
}

macro_rules! declare_resource {
    ( $CMP:ident ) => {
        impl $crate::resource::resource::ResourceIndex for $CMP {
            fn index() -> usize {
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