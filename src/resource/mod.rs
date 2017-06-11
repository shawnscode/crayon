pub mod errors;
pub mod archive;
pub mod cache;
pub mod texture;

pub use self::archive::{Read, Seek, Archive, FilesystemArchive, ZipArchive, ArchiveCollection};
pub use self::cache::Cache;
pub use self::texture::Texture;

use std::any::Any;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;
use self::errors::*;

pub trait ResourceItem: ::std::fmt::Debug {
    /// Create resource from raw bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized;

    /// Return internal memory usages of resource in bytes.
    fn size(&self) -> usize;
}

pub trait ResourceItemIndex {
    fn index() -> usize;
}

///
pub struct Resources {
    archives: ArchiveCollection,
    caches: Vec<Option<Box<Any>>>,
    buf: Vec<u8>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            archives: ArchiveCollection::new(),
            caches: Vec::new(),
            buf: Vec::new(),
        }
    }

    /// Return mutable reference to `ArchiveCollection`.
    pub fn archives(&mut self) -> &mut ArchiveCollection {
        &mut self.archives
    }

    /// Register a `Cache<T>` with specified cache capacity.
    pub fn register_cache<T>(&mut self, size: usize)
        where T: ResourceItem + ResourceItemIndex + 'static
    {
        if T::index() >= self.caches.len() {
            for _ in self.caches.len()..(T::index() + 1) {
                self.caches.push(None);
            }
        }

        if let Some(_) = self.caches[T::index()] {
            return;
        }

        self.caches[T::index()] = Some(Box::new(Cache::<RwLock<T>>::new(size)));
    }

    pub fn loads<T, P>(&mut self, path: P) -> Result<Arc<RwLock<T>>>
        where P: AsRef<Path>, T: ResourceItem + ResourceItemIndex + 'static
    {
        if let Some(cache) = self.cache_mut::<T>() {
            if let Some(v) = cache.get(path.as_ref()) {
                return Ok(v.clone());
            }
        }

        self.archives.read(path.as_ref(), &mut self.buf)?;
        let resource = T::from_bytes(self.buf.as_slice())?;
        let size = resource.size();

        self.buf.clear();
        let rc = Arc::new(RwLock::new(resource));
        if let Some(cache) = self.cache_mut::<T>() {
            cache.insert(path.as_ref(), size, rc.clone());
        }

        Ok(rc)
    }

    fn cache_mut<T>(&mut self) -> Option<&mut Cache<RwLock<T>>>
        where T: ResourceItemIndex + ResourceItem + 'static
    {
        if let Some(element) = self.caches.get_mut(T::index()) {
            if let Some(ref mut cache) = *element {
                return Some(cache.downcast_mut::<Cache<RwLock<T>>>().unwrap());
            }
        }

        None
    }
}

lazy_static! {
    /// Lazy initialized id of component. Which produces a continuous index address.
    #[doc(hidden)]
    static ref _INDEX: AtomicUsize = AtomicUsize::new(0);
}

macro_rules! declare_resource {
    ( $CMP:ident ) => {
        impl $crate::resource::ResourceItemIndex for $CMP {
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