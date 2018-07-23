use std::any::TypeId;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};

use sched::latch::{LatchProbe, LatchWaitProbe};
use utils::handle::Handle;
use utils::hash_value::HashValue;

use super::errors::*;
use super::ResourceHandle;

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
    pub fn set(&self, v: Result<()>) {
        {
            let mut guard = self.m.lock().unwrap();
            *guard = Promise::Ok(v);
        }

        self.v.notify_all();
    }

    #[inline]
    pub fn take(&self) -> Result<()> {
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
struct SchemaLocation {
    schema: TypeId,
    location: HashValue<Path>,
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
    locations: HashMap<SchemaLocation, SchemaHandle>,
    entries: HashMap<SchemaHandle, Entry>,
}

impl Registery {
    pub fn new() -> Self {
        Registery {
            locations: HashMap::new(),
            entries: HashMap::new(),
        }
    }

    pub fn create<H>(&mut self, location: HashValue<Path>, handle: H) -> Arc<PromiseLatch>
    where
        H: ResourceHandle,
    {
        let schema = TypeId::of::<H>();
        let latch = Arc::new(PromiseLatch::new());

        let sl = SchemaLocation {
            schema: schema,
            location: location,
        };

        let sh = SchemaHandle {
            schema: schema,
            handle: handle.into(),
        };

        let v = Entry {
            rc: 1,
            latch: latch.clone(),
        };

        self.locations.insert(sl, sh);
        self.entries.insert(sh, v);

        latch
    }

    pub fn try_inc_rc<H>(&mut self, location: HashValue<Path>) -> Option<H>
    where
        H: ResourceHandle,
    {
        let schema = TypeId::of::<H>();
        let sl = SchemaLocation {
            schema: schema,
            location: location,
        };

        if let Some(k) = self.locations.get(&sl) {
            let v = self.entries.get_mut(k).unwrap();
            v.rc += 1;
            return Some(k.handle.into());
        }

        None
    }

    pub fn try_dec_rc<H>(&mut self, handle: H) -> bool
    where
        H: ResourceHandle,
    {
        let schema = TypeId::of::<H>();
        let sh = SchemaHandle {
            schema: schema,
            handle: handle.into(),
        };

        if let Some(v) = self.entries.get_mut(&sh) {
            v.rc -= 1;
            if v.rc <= 0 {
                return true;
            }
        }

        false
    }

    pub fn try_promise<H>(&self, handle: H) -> Option<Arc<PromiseLatch>>
    where
        H: ResourceHandle,
    {
        let schema = TypeId::of::<H>();
        let sh = SchemaHandle {
            schema: schema,
            handle: handle.into(),
        };

        self.entries.get(&sh).map(|v| v.latch.clone())
    }
}
