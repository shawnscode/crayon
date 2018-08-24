use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use uuid::Uuid;

use errors::*;
use sched::latch::{LatchProbe, LatchWaitProbe};
use sched::ScheduleSystemShared;
use utils::handle::Handle;
use utils::hash_value::HashValue;

use super::location::Location;
use super::manifest;
use super::vfs::{VFSDriver, VFS};
use super::{ResourceHandle, ResourceLoader};

enum Promise {
    NotReady,
    Ok(Result<()>),
}

pub struct PromiseLatch {
    m: Mutex<Promise>,
    v: Condvar,
}

impl PromiseLatch {
    #[inline]
    fn new() -> Self {
        PromiseLatch {
            m: Mutex::new(Promise::NotReady),
            v: Condvar::new(),
        }
    }

    #[inline]
    pub(crate) fn set(&self, v: Result<()>) {
        {
            let mut guard = self.m.lock().unwrap();
            *guard = Promise::Ok(v);
        }

        self.v.notify_all();
    }

    #[inline]
    pub(crate) fn take(&self) -> Result<()> {
        let mut guard = self.m.lock().unwrap();
        if let Promise::Ok(v) = ::std::mem::replace(&mut *guard, Promise::Ok(Ok(()))) {
            v
        } else {
            unreachable!();
        }
    }
}

impl LatchProbe for PromiseLatch {
    fn is_set(&self) -> bool {
        let guard = self.m.lock().unwrap();
        if let Promise::NotReady = *guard {
            false
        } else {
            true
        }
    }
}

impl LatchWaitProbe for PromiseLatch {
    fn wait(&self) {
        let mut guard = self.m.lock().unwrap();
        while let Promise::NotReady = *guard {
            guard = self.v.wait(guard).unwrap();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SchemaHandle {
    schema: TypeId,
    handle: Handle,
}

struct Entry {
    rc: u32,
    latch: Arc<PromiseLatch>,
}

pub struct Registery {
    sched: Arc<ScheduleSystemShared>,
    locs: HashMap<Uuid, SchemaHandle>,
    entries: HashMap<SchemaHandle, Entry>,

    driver: VFSDriver,
    manifest: HashMap<Uuid, HashValue<str>>,
    remaps: HashMap<HashValue<Path>, Uuid>,
}

impl Registery {
    pub fn new(sched: Arc<ScheduleSystemShared>) -> Self {
        Registery {
            sched: sched,
            locs: HashMap::new(),
            entries: HashMap::new(),
            driver: VFSDriver::new(),
            manifest: HashMap::new(),
            remaps: HashMap::new(),
        }
    }

    pub fn mount<F>(&mut self, name: &str, vfs: F) -> Result<()>
    where
        F: VFS + 'static,
    {
        info!("Mounts virtual file system {}.", name);

        let mut file = vfs.read(manifest::NAME.as_ref())?;
        let name = name.into();

        let man = manifest::Manifest::load(&mut file)?;
        for v in &man.items {
            self.manifest.insert(v.uuid, name);
            self.remaps.insert(v.location, v.uuid);
        }

        self.driver.mount(name, vfs)
    }

    pub fn load_from<T>(&mut self, loader: Arc<Any + Send + Sync>, location: Location) -> Result<T>
    where
        T: ResourceHandle,
    {
        let (fs, uuid) = match location {
            Location::Uuid(uuid) => {
                if let Some(fs) = self.manifest.get(&uuid) {
                    (*fs, uuid)
                } else {
                    bail!("Uuid {:X} not found.", uuid);
                }
            }

            Location::Name(fs, file) => {
                let hash: HashValue<Path> = (&file).into();
                if let Some(uuid) = self.remaps.get(&hash) {
                    (fs, *uuid)
                } else {
                    bail!("File {:?} not found.", file);
                }
            }
        };

        if let Some(k) = self.locs.get(&uuid) {
            let v = self.entries.get_mut(k).unwrap();
            v.rc += 1;
            return Ok(k.handle.into());
        }

        let handle = {
            // FIXME: `rc_downcast`.
            let dc: &T::Loader = (loader.as_ref() as &Any).downcast_ref().unwrap();
            dc.create()?
        };

        let sh = SchemaHandle {
            schema: TypeId::of::<T>(),
            handle: handle.into(),
        };

        let latch = Arc::new(PromiseLatch::new());
        let v = Entry {
            rc: 1,
            latch: latch.clone(),
        };

        self.locs.insert(uuid, sh);
        self.entries.insert(sh, v);

        let path = format!("{:X}", uuid.simple());
        let mut file = self.driver.read(fs, path.as_ref())?;

        self.sched.spawn(move || {
            let dc: &T::Loader = (loader.as_ref() as &Any).downcast_ref().unwrap();
            latch.set(dc.load(handle, &mut file));
        });

        Ok(handle)
    }

    pub fn promise<T>(&self, handle: T) -> Option<Arc<PromiseLatch>>
    where
        T: ResourceHandle,
    {
        let sh = SchemaHandle {
            schema: TypeId::of::<T>(),
            handle: handle.into(),
        };

        self.entries.get(&sh).map(|v| v.latch.clone())
    }

    pub fn unload<T>(&mut self, loader: Arc<Any + Send + Sync>, handle: T) -> Result<()>
    where
        T: ResourceHandle,
    {
        let sh = SchemaHandle {
            schema: TypeId::of::<T>(),
            handle: handle.into(),
        };

        if let Some(v) = self.entries.get_mut(&sh) {
            v.rc -= 1;
            if v.rc <= 0 {
                let dc: &T::Loader = (loader.as_ref() as &Any).downcast_ref().unwrap();
                return dc.delete(handle);
            }
        }

        Ok(())
    }
}
