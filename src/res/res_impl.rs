use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};

use sched;
use sched::latch::{Latch, LockLatch};
use utils::handle::Handle;
use utils::hash_value::HashValue;

use super::errors::*;

pub trait ResourceHandle: Into<Handle> + From<Handle> + Copy + Send {
    type Loader: ResourceLoader<Handle = Self>;
}

pub trait ResourceLoader: Send + Sync + Sized + 'static {
    const SCHEMA: &'static str;
    type Handle: ResourceHandle<Loader = Self>;

    fn create(&self) -> Result<Self::Handle>;
    fn load(&self, handle: Self::Handle, file: &mut dyn Read) -> Result<()>;
    fn delete(&self, handle: Self::Handle) -> Result<()>;
}

pub struct ResourceSystem {
    loaders: Arc<RwLock<HashMap<HashValue<str>, Arc<Any + Send + Sync>>>>,
    shared: Arc<ResourceSystemShared>,
}

impl ResourceSystem {
    pub fn new() -> Result<Self> {
        let loaders = Arc::new(RwLock::new(HashMap::new()));

        let shared = Arc::new(ResourceSystemShared {
            loaders: loaders.clone(),
            items: RwLock::new(ResourceTable {
                handles: HashMap::new(),
                entries: HashMap::new(),
            }),
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
        let schema = T::SCHEMA.into();
        self.loaders
            .write()
            .unwrap()
            .insert(schema, Arc::new(loader));
    }

    pub fn shared(&self) -> Arc<ResourceSystemShared> {
        self.shared.clone()
    }
}

pub struct ResourceSystemShared {
    loaders: Arc<RwLock<HashMap<HashValue<str>, Arc<Any + Send + Sync>>>>,
    items: RwLock<ResourceTable>,
}

impl ResourceSystemShared {
    pub fn load<T>(&self, location: &Path) -> Result<T>
    where
        T: ResourceHandle + 'static,
    {
        let schema = T::Loader::SCHEMA.into();
        let any = self.loaders.read().unwrap().get(&schema).unwrap().clone();

        let k = SchemaLocation {
            schema: schema,
            location: location.into(),
        };

        let (handle, latch) = {
            let mut items = self.items.write().unwrap();

            let handle = {
                if let Some(v) = items.entries.get_mut(&k) {
                    v.rc += 1;
                    return Ok(v.handle.into());
                }

                let loader: &T::Loader = (any.as_ref() as &Any).downcast_ref().unwrap();
                loader.create()?
            };

            let latch = Arc::new(LockLatch::new());

            let v = Ref {
                rc: 1,
                handle: handle.into(),
                latch: latch.clone(),
            };

            let kk = SchemaHandle {
                schema: schema,
                handle: handle.into(),
            };

            items.handles.insert(kk, k);
            items.entries.insert(k, v);

            (handle, latch)
        };

        let mut file = fs::File::open(location)?;
        sched::spawn(move || {
            let loader: &T::Loader = (any.as_ref() as &Any).downcast_ref().unwrap();
            loader.load(handle, &mut file).unwrap();
            latch.set();
        });

        Ok(handle)
    }

    pub fn wait<T>(&self, handle: T) -> Result<()>
    where
        T: ResourceHandle,
    {
        Ok(())
    }

    pub fn unload<T>(&self, handle: T) -> Result<()>
    where
        T: ResourceHandle,
    {
        let schema = T::Loader::SCHEMA.into();
        let k = SchemaHandle {
            schema: schema,
            handle: handle.into(),
        };

        let v = {
            let mut items = self.items.write().unwrap();
            if items.dec_rc(k) {
                let kk = items.handles.get(&k).cloned().unwrap();
                items.handles.remove(&k);
                Some(items.entries.remove(&kk).unwrap())
            } else {
                None
            }
        };

        if let Some(v) = v {
            let any = self.loaders.read().unwrap().get(&schema).unwrap().clone();
            let loader: &T::Loader = (any.as_ref() as &Any).downcast_ref().unwrap();
            loader.delete(v.handle.into())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SchemaLocation {
    schema: HashValue<str>,
    location: HashValue<Path>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SchemaHandle {
    schema: HashValue<str>,
    handle: Handle,
}

struct Ref {
    rc: u32,
    handle: Handle,
    latch: Arc<LockLatch>,
}

struct ResourceTable {
    handles: HashMap<SchemaHandle, SchemaLocation>,
    entries: HashMap<SchemaLocation, Ref>,
}

impl ResourceTable {
    fn dec_rc(&mut self, k: SchemaHandle) -> bool {
        if let Some(kk) = self.handles.get(&k) {
            if let Some(v) = self.entries.get_mut(&kk) {
                v.rc -= 1;
                if v.rc == 0 {
                    return true;
                }
            }
        }

        false
    }
}
