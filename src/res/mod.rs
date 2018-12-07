//! The `ResourceSystem` provides standardized interface to load data asynchronously from various
//! filesystem, and some utilities for modules to implement their own local resource management.
//!
//! To understand how to properly manage data in _crayon_, its important to understand how _crayon_
//! identifies and serializes data. The first key point is the distinction between _asset_s and
//! _resource_s.
//!
//! # Asset
//!
//! An asset is a file on disk, such like textures, 3D models, or audio clips. Since assets might
//! be be modified by artiest continuous, its usually stored in formats which could producing and
//! editing by authoring tools directly. Its always trival and error-prone to load and managet
//! assets directly at runtime.
//!
//! # Resource
//!
//! A _resource_ is a abstraction of some `piece of data` that are fully prepared for using at runtime.
//! We are providing a command line tool [crayon-cli](https://github.com/shawnscode/crayon-tools) that
//! automatically compiles assets into resources for runtime.
//!
//! ## UUID
//!
//! An asset can produces multiple resources eventually. For example, `FBX` file can have multiple
//! models, and it can also contains a spatial description of objects. For every resource that an
//! asset might produces, a universal-uniqued id (UUID) is assigned to it. UUIDs are stored in .meta
//! files. These .meta files are generated when _crayon-cli_ first imports an asset, and are stored
//! in the same directory as the asset.
//!
//! # Virtual Filesystem (VFS)
//!
//! The `ResourceSystem` allows load data asynchronously from web servers, the local host
//! filesystem, or other places if extended by pluggable `VFS`.
//!
//! The `VFS` trait has a pretty simple interface, since it should focus on games that load
//! data asynchronously. A trival `Directory` is provided to supports local host filesystem.
//! And it should be easy to add features like compression and encrpytion.
//!
//! ## Manifest
//!
//! Every VFS should have a `Manifest` file which could be used to locate resources in actual path
//! from general UUID or readable identifier. The `Manifest` file is generated after the build
//! process of `crayon-cli`.
//!

pub mod manifest;
pub mod request;
pub mod shortcut;
pub mod url;
pub mod utils;
pub mod vfs;

pub mod prelude {
    pub use super::utils::prelude::ResourceState;
    pub use super::ResourceParams;
}

mod system;

use std::sync::Arc;

use failure::ResultExt;
use uuid::Uuid;

use crate::sched::prelude::{CountLatch, Latch};

use self::ins::{ctx, CTX};
use self::request::{Request, Response};
use self::shortcut::ShortcutResolver;
use self::system::ResourceSystem;
use self::vfs::SchemaResolver;

#[derive(Debug, Clone)]
pub struct ResourceParams {
    pub shortcuts: ShortcutResolver,
    pub schemas: SchemaResolver,
    pub dirs: Vec<String>,
}

impl Default for ResourceParams {
    fn default() -> Self {
        let mut params = ResourceParams {
            shortcuts: ShortcutResolver::new(),
            schemas: SchemaResolver::new(),
            dirs: Vec::new(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        params.schemas.add("file", self::vfs::dir::Dir::new());
        #[cfg(target_arch = "wasm32")]
        params.schemas.add("http", self::vfs::http::Http::new());

        params
    }
}

/// Setup the resource system.
pub(crate) unsafe fn setup(params: ResourceParams) -> Result<(), failure::Error> {
    debug_assert!(CTX.is_null(), "duplicated setup of resource system.");

    let ctx = ResourceSystem::new(params)?;
    CTX = Box::into_raw(Box::new(ctx));
    Ok(())
}

/// Attach manifests to this registry.
pub(crate) fn load_manifests(dirs: Vec<String>) -> Result<Arc<CountLatch>, failure::Error> {
    let latch = Arc::new(CountLatch::new());

    for v in dirs {
        let clone = latch.clone();
        clone.increment();

        let prefix = v.clone();
        ctx().load_manifest_with_callback(v, move |rsp| {
            let bytes = rsp
                .with_context(|_| format!("Failed to load manifest from {}", prefix))
                .unwrap();

            let mut cursor = std::io::Cursor::new(bytes);
            ctx().attach(&prefix, &mut cursor).unwrap();
            clone.set();
        })?;
    }

    latch.set();
    Ok(latch)
}

/// Discard the resource system.
pub(crate) unsafe fn discard() {
    if CTX.is_null() {
        return;
    }

    drop(Box::from_raw(CTX as *mut ResourceSystem));
    CTX = std::ptr::null();
}

/// Checks if the resource system is enabled.
#[inline]
pub fn valid() -> bool {
    unsafe { !CTX.is_null() }
}

/// Resolve shortcuts in the provided string recursively and return None if not exists.
#[inline]
pub fn resolve<T: AsRef<str>>(url: T) -> Option<String> {
    ctx().resolve(url)
}

/// Return the UUID of resource located at provided path, and return None if not exists.
#[inline]
pub fn find<T: AsRef<str>>(filename: T) -> Option<Uuid> {
    ctx().find(filename)
}

/// Checks if the resource exists in this registry.
#[inline]
pub fn exists(uuid: Uuid) -> bool {
    ctx().exists(uuid)
}

/// Loads file asynchronously with response callback.
#[inline]
pub fn load_with_callback<T>(uuid: Uuid, func: T) -> Result<(), failure::Error>
where
    T: FnOnce(Response) + Send + 'static,
{
    ctx().load_with_callback(uuid, func)
}

/// Loads file asynchronously with response callback.
#[inline]
pub fn load_from_with_callback<T1, T2>(filename: T1, func: T2) -> Result<(), failure::Error>
where
    T1: AsRef<str>,
    T2: FnOnce(Response) + Send + 'static,
{
    ctx().load_from_with_callback(filename, func)
}

/// Loads file asynchronously. This method will returns a `Request` object immediatedly,
/// its user's responsibility to store the object and frequently check it for completion.
pub fn load(uuid: Uuid) -> Result<Request, failure::Error> {
    ctx().load(uuid)
}

/// Loads file asynchronously. This method will returns a `Request` object immediatedly,
/// its user's responsibility to store the object and frequently check it for completion.
pub fn load_from<T: AsRef<str>>(filename: T) -> Result<Request, failure::Error> {
    ctx().load_from(filename)
}

mod ins {
    use super::system::ResourceSystem;

    pub static mut CTX: *const ResourceSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static ResourceSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "resource system has not been initialized properly."
            );

            &*CTX
        }
    }
}
