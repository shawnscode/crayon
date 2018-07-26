//! # Resource
//!
//! A resource is a very slim proxy object that adds a standardized interface for creation,
//! destruction, sharing and lifetime management to some external object or generally
//! ‘piece of data'.
//!
//! we are using a unique `Handle` object to represent a resource object safely. This approach
//! has several advantages, since it helps for saving state externally. E.G.:
//!
//! 1. It allows for the resource to be destroyed without leaving dangling pointers.
//! 2. Its perfectly safe to store and share the `Handle` even the underlying resource is
//! loading on the background thread.
//!
//! In some systems, actual resource objects are private and opaque, application will usually
//! not have direct access to a resource object in form of reference.

pub mod byteorder;
pub mod errors;
pub mod location;
pub mod manifest;
pub mod vfs;

pub mod prelude {
    pub use super::vfs::DiskFS;
    pub use super::{ResourceHandle, ResourceLoader, ResourceSystem, ResourceSystemShared};
}

mod registery;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, RwLock};

use sched::ScheduleSystemShared;
use utils::handle::Handle;

use self::errors::*;
use self::vfs::VFS;

pub trait ResourceHandle: Into<Handle> + From<Handle> + Copy + Send + 'static {
    type Loader: ResourceLoader<Handle = Self>;
}

pub trait ResourceLoader: Send + Sync + Sized + 'static {
    type Handle: ResourceHandle<Loader = Self>;

    fn create(&self) -> Result<Self::Handle>;
    fn load(&self, handle: Self::Handle, file: &mut dyn Read) -> Result<()>;
    fn delete(&self, handle: Self::Handle) -> Result<()>;
}

pub struct ResourceSystem {
    loaders: Arc<RwLock<HashMap<TypeId, Arc<Any + Send + Sync>>>>,
    registery: Arc<RwLock<registery::Registery>>,
    shared: Arc<ResourceSystemShared>,
}

impl ResourceSystem {
    pub fn new(sched: Arc<ScheduleSystemShared>) -> Result<Self> {
        let loaders = Arc::new(RwLock::new(HashMap::new()));
        let registery = Arc::new(RwLock::new(registery::Registery::new(sched.clone())));

        let shared = Arc::new(ResourceSystemShared {
            sched: sched,
            loaders: loaders.clone(),
            registery: registery.clone(),
        });

        Ok(ResourceSystem {
            shared: shared,
            loaders: loaders,
            registery: registery,
        })
    }

    pub fn register<T>(&self, loader: T)
    where
        T: ResourceLoader,
    {
        self.loaders
            .write()
            .unwrap()
            .insert(TypeId::of::<T::Handle>(), Arc::new(loader));
    }

    pub fn mount<F>(&mut self, name: &str, vfs: F) -> Result<()>
    where
        F: VFS + 'static,
    {
        self.registery.write().unwrap().mount(name, vfs)
    }

    pub fn shared(&self) -> Arc<ResourceSystemShared> {
        self.shared.clone()
    }

    pub fn advance(&self) {}
}

pub struct ResourceSystemShared {
    loaders: Arc<RwLock<HashMap<TypeId, Arc<Any + Send + Sync>>>>,
    registery: Arc<RwLock<registery::Registery>>,
    sched: Arc<ScheduleSystemShared>,
}

impl ResourceSystemShared {
    /// Loads a resource from location.
    pub fn load<T>(&self, uri: &str) -> Result<T>
    where
        T: ResourceHandle + 'static,
    {
        self.load_from(location::Location::from_str(uri)?)
    }

    pub fn load_from<T>(&self, location: location::Location) -> Result<T>
    where
        T: ResourceHandle + 'static,
    {
        let schema = TypeId::of::<T>();
        let loader = self.loaders.read().unwrap().get(&schema).unwrap().clone();
        self.registery.write().unwrap().load_from(loader, location)
    }

    /// Blocks current thread until loader is finished.
    pub fn wait<T>(&self, handle: T) -> Result<()>
    where
        T: ResourceHandle,
    {
        if let Some(promise) = self.registery.read().unwrap().promise(handle) {
            self.sched.wait_until(promise.as_ref());
            promise.take()
        } else {
            Ok(())
        }
    }

    /// Unloads a resource when associated with `Handle`.
    pub fn unload<T>(&self, handle: T) -> Result<()>
    where
        T: ResourceHandle,
    {
        let schema = TypeId::of::<T>();
        let loader = self.loaders.read().unwrap().get(&schema).unwrap().clone();
        self.registery.write().unwrap().unload(loader, handle)
    }
}
