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
//! ## Location
//!
//! User could locates a `resource` with `Location`. A `Location` consists of two parts, a virtual
//! filesystem prefix and a readable identifier of resource.
//!
//! For example, let's say a game has all its textures in a special asset subdirectory called
//! `resources/textures`. The game would define an virtual filesystem called `res:` pointing to that
//! directory, and texture location would be defined like this:
//!
//! ```sh
//! "res:textures/crate.png"
//! ```
//!
//! Before those textures are actually loaded, the `res:` prefix is replaced with an absolute directory
//! location, and the readable path is replaced with the actual local path (which are usually the hex
//! representation of UUID).
//!
//! ```sh
//! "res:textures/crate.png" => "/Applications/My Game/resources/textures/2943B9386A274730A50702A904F384D5"
//! ```
//!
//! This makes it easier to load data from other places than the local hard disc, like web servers,
//! communicating with HTTP REST services or implementing more exotic ways to load data.
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
//! # Registry
//!
//! The `Registry` is a standardized resource manager that defines a set of interface for creation,
//! destruction, sharing and lifetime management. It is used in all the built-in crayon modules.
//!
//! ## Handle
//!
//! We are using a unique `Handle` object to represent a resource object safely. This approach
//! has several advantages, since it helps for saving state externally. E.G.:
//!
//! 1. It allows for the resource to be destroyed without leaving dangling pointers.
//! 2. Its perfectly safe to store and share the `Handle` even the underlying resource is
//! loading on the background thread.
//!
//! In some systems, actual resource objects are private and opaque, application will usually
//! not have direct access to a resource object in form of reference.
//!
//! ## Ownership & Lifetime
//!
//! For the sake of simplicity, the refenerce-counting technique is used for providing shared ownership
//! of a resource.
//!
//! Everytime you create a resource at runtime, the `Registry` will increases the reference count of
//! the resource by 1. And when you are done with the resource, its the user's responsibility to
//! drop the ownership of the resource. And when the last ownership to a given resource is dropped,
//! the corresponding resource is also destroyed.
//!

pub mod location;
use self::location::Location;

pub mod promise;
use self::promise::Promise;

pub mod registry;
pub mod vfs;

pub mod prelude {
    pub use super::location::Location;
    pub use super::promise::Promise;
    pub use super::vfs::Directory;
    pub use super::{ResourceSystem, ResourceSystemShared};
}

use std::sync::{Arc, RwLock};
use uuid::Uuid;

use self::vfs::{VFSDriver, VFS};

use errors::*;
use sched::ScheduleSystemShared;
use utils::FastHashMap;

/// The `ResourceSystem` Takes care of loading data asynchronously through pluggable filesystems.
pub struct ResourceSystem {
    driver: Arc<RwLock<VFSDriver>>,
    shared: Arc<ResourceSystemShared>,
}

impl ResourceSystem {
    /// Creates a new `ResourceSystem`.
    pub fn new(sched: Arc<ScheduleSystemShared>) -> Result<Self> {
        let driver = Arc::new(RwLock::new(VFSDriver::new()));

        let shared = Arc::new(ResourceSystemShared {
            driver: driver.clone(),
            sched: sched,
            bufs: Arc::new(RwLock::new(Vec::new())),
            promises: Arc::new(RwLock::new(FastHashMap::default())),
        });

        Ok(ResourceSystem {
            driver: driver,
            shared: shared,
        })
    }

    /// Mount a file-system drive with identifier.
    pub fn mount<T, F>(&mut self, name: T, vfs: F) -> Result<()>
    where
        T: AsRef<str>,
        F: VFS + 'static,
    {
        let name = name.as_ref();
        info!("Mounts virtual file system {}.", name);
        self.driver.write().unwrap().mount(name, vfs)
    }

    /// Returns the multi-thread friendly parts of `ResourceSystem`.
    pub fn shared(&self) -> Arc<ResourceSystemShared> {
        self.shared.clone()
    }
}

pub trait Loader: Send + Sync + 'static {
    fn load(&self, file: &[u8]) -> Result<()>;
}

pub struct ResourceSystemShared {
    driver: Arc<RwLock<VFSDriver>>,
    sched: Arc<ScheduleSystemShared>,

    bufs: Arc<RwLock<Vec<Vec<u8>>>>,
    promises: Arc<RwLock<FastHashMap<Uuid, Arc<Promise>>>>,
}

impl ResourceSystemShared {
    /// Redirects a readabke location into universe-uniqued identifier (Uuid).
    pub fn redirect(&self, location: Location) -> Option<Uuid> {
        self.driver
            .read()
            .unwrap()
            .vfs(location.vfs())
            .and_then(|vfs| vfs.redirect(location.filename()))
    }

    /// Loads a resource at readable location asynchronously.
    pub fn load_from<T: Loader>(&self, loader: T, location: Location) -> Result<Arc<Promise>> {
        let uuid = self.redirect(location).ok_or_else(|| {
            format_err!(
                "Undefined virtual filesystem with identifier {}.",
                location.vfs()
            )
        })?;

        self.load_from_uuid(loader, uuid)
    }

    /// Loads a resource with uuid asynchronously.
    pub fn load_from_uuid<T: Loader>(&self, loader: T, uuid: Uuid) -> Result<Arc<Promise>> {
        let vfs = self
            .driver
            .read()
            .unwrap()
            .vfs_from_uuid(uuid)
            .ok_or_else(|| format_err!("Undefined uuid with {}", uuid))?;

        let latch = {
            let mut promises = self.promises.write().unwrap();
            if promises.contains_key(&uuid) {
                bail!("Circular reference of resource {} found!", uuid);
            }

            let latch = Arc::new(Promise::new());
            promises.insert(uuid, latch.clone());
            latch
        };

        let tx = latch.clone();
        let bufs = self.bufs.clone();
        let promises = self.promises.clone();

        self.sched.spawn(move || {
            let mut bytes = bufs.write().unwrap().pop().unwrap_or(Vec::new());
            let uri = vfs.locate(uuid).unwrap();

            if let Err(err) = vfs.read_to_end(&uri, &mut bytes) {
                tx.set(Err(err));
            } else {
                tx.set(loader.load(&bytes));
            }

            promises.write().unwrap().remove(&uuid);
            bytes.clear();
            bufs.write().unwrap().push(bytes);
        });

        Ok(latch)
    }

    /// Blocks current thread until the loading process of resource `uuid` finished.
    pub fn wait_until(&self, uuid: Uuid) -> Result<()> {
        let promise = self.promises.read().unwrap().get(&uuid).cloned();
        if let Some(promise) = promise {
            self.sched.wait_until(promise.as_ref());
            promise.take()
        } else {
            Ok(())
        }
    }
}
