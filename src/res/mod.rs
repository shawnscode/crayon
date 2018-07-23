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

mod registery;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};

use sched;
use utils::handle::Handle;

use self::errors::*;
use self::registery::Registery;

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
    shared: Arc<ResourceSystemShared>,
}

impl ResourceSystem {
    pub fn new(sched: Arc<sched::ScheduleSystemShared>) -> Result<Self> {
        let loaders = Arc::new(RwLock::new(HashMap::new()));

        let shared = Arc::new(ResourceSystemShared {
            sched: sched,
            loaders: loaders.clone(),
            registery: RwLock::new(Registery::new()),
        });

        Ok(ResourceSystem {
            loaders: loaders,
            shared: shared,
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

    pub fn shared(&self) -> Arc<ResourceSystemShared> {
        self.shared.clone()
    }
}

pub struct ResourceSystemShared {
    loaders: Arc<RwLock<HashMap<TypeId, Arc<Any + Send + Sync>>>>,
    sched: Arc<sched::ScheduleSystemShared>,
    registery: RwLock<Registery>,
}

impl ResourceSystemShared {
    /// Loads a resource from location.
    pub fn load<T>(&self, location: &Path) -> Result<T>
    where
        T: ResourceHandle + 'static,
    {
        let schema = TypeId::of::<T>();
        let loc = location.into();
        let loader = self.loaders.read().unwrap().get(&schema).unwrap().clone();

        let (handle, latch) = {
            let mut registery = self.registery.write().unwrap();
            if let Some(handle) = registery.try_inc_rc(loc) {
                return Ok(handle);
            }

            let r: &T::Loader = (loader.as_ref() as &Any).downcast_ref().unwrap();
            let handle = r.create()?;
            let latch = registery.create(loc, handle);
            (handle, latch)
        };

        let mut file = fs::File::open(location)?;
        self.sched.spawn(move || {
            let loader: &T::Loader = (loader.as_ref() as &Any).downcast_ref().unwrap();
            let v = loader.load(handle, &mut file);
            latch.set(v);
        });

        Ok(handle)
    }

    /// Blocks current thread until loader is finished.
    pub fn wait<T>(&self, handle: T) -> Result<()>
    where
        T: ResourceHandle,
    {
        if let Some(promise) = self.registery.read().unwrap().try_promise(handle) {
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

        if self.registery.write().unwrap().try_dec_rc(handle) {
            let any = self.loaders.read().unwrap().get(&schema).unwrap().clone();
            let loader: &T::Loader = (any.as_ref() as &Any).downcast_ref().unwrap();
            loader.delete(handle)?;
        }

        Ok(())
    }
}
