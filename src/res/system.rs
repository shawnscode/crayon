use std::sync::{Arc, Mutex};

use uuid::Uuid;

use application::prelude::{LifecycleListener, LifecycleListenerHandle};
use sched::prelude::{Latch, LockCountLatch};

use super::manifest::ManfiestResolver;
use super::request::{Request, RequestQueue, Response};
use super::shortcut::ShortcutResolver;
use super::url::Url;
use super::vfs::SchemaResolver;
use super::ResourceParams;

pub struct ResourceSystem {
    shortcut: ShortcutResolver,
    manifest: ManfiestResolver,
    schemas: SchemaResolver,
    requests: Arc<Mutex<RequestQueue>>,
    lifecycle: LifecycleListenerHandle,
}

struct Lifecycle {
    requests: Arc<Mutex<RequestQueue>>,
}

impl LifecycleListener for Lifecycle {
    fn on_update(&mut self) -> Result<(), failure::Error> {
        self.requests.lock().unwrap().advance();
        Ok(())
    }
}

impl Drop for ResourceSystem {
    fn drop(&mut self) {
        crate::application::detach(self.lifecycle);
    }
}

impl ResourceSystem {
    pub fn new(params: ResourceParams) -> Result<Self, failure::Error> {
        debug_assert!(crate::application::valid(), "");

        let latch = Arc::new(LockCountLatch::new());
        let mut requests = Vec::new();

        for v in &params.dirs {
            let prefix = params
                .shortcuts
                .resolve(v)
                .ok_or_else(|| format_err!("Could not resolve manifest dir: {}.", v))?;
            let url = Url::new(format!("{}{}", prefix, super::manifest::NAME))?;

            let vfs = params.schemas.locate(url.schema())?;
            let state = Request::latch();
            let req = Request::new(state.clone());

            let clone = latch.clone();
            clone.increment();

            crate::sched::spawn(move || {
                vfs.request(&url, state);
                clone.set();
            });

            requests.push((prefix, req));
        }

        latch.set();
        crate::sched::wait_until(latch.as_ref());

        let mut manifest = ManfiestResolver::new();
        for (prefix, mut v) in requests.drain(..) {
            v.wait();

            match *v.response().unwrap() {
                Ok(ref bytes) => {
                    let mut cursor = std::io::Cursor::new(bytes);
                    manifest.add(prefix.as_ref(), &mut cursor)?;
                }
                Err(ref error) => panic!("Could not load manifest from {}.\n{}", prefix, error),
            }
        }

        let requests = Arc::new(Mutex::new(RequestQueue::new()));
        let sys = ResourceSystem {
            shortcut: params.shortcuts,
            schemas: params.schemas,
            manifest: manifest,
            requests: requests.clone(),
            lifecycle: crate::application::attach(Lifecycle { requests }),
        };

        Ok(sys)
    }

    /// Resolve shortcuts in the provided string recursively and return None if not exists.
    #[inline]
    pub fn resolve<T: AsRef<str>>(&self, url: T) -> Option<String> {
        self.shortcut.resolve(url.as_ref())
    }

    /// Return the UUID of resource located at provided path, and return None if not exists.
    #[inline]
    pub fn find<T: AsRef<str>>(&self, filename: T) -> Option<Uuid> {
        let filename = filename.as_ref();
        self.shortcut
            .resolve(filename)
            .and_then(|url| self.manifest.find(&url))
    }

    /// Checks if the resource exists in this registry.
    #[inline]
    pub fn exists(&self, uuid: Uuid) -> bool {
        self.manifest.contains(uuid)
    }

    /// Loads file asynchronously with response callback.
    #[inline]
    pub fn load_with_callback<T>(&self, uuid: Uuid, func: T) -> Result<(), failure::Error>
    where
        T: Fn(&Response) + 'static,
    {
        let req = self.load(uuid)?;
        self.requests.lock().unwrap().add(req, func);
        Ok(())
    }

    /// Loads file asynchronously with response callback.
    #[inline]
    pub fn load_from_with_callback<T1, T2>(
        &self,
        filename: T1,
        func: T2,
    ) -> Result<(), failure::Error>
    where
        T1: AsRef<str>,
        T2: Fn(&Response) + 'static,
    {
        let req = self.load_from(filename)?;
        self.requests.lock().unwrap().add(req, func);
        Ok(())
    }

    /// Loads file asynchronously. This method will returns a `Request` object immediatedly,
    /// its user's responsibility to store the object and frequently check it for completion.
    pub fn load(&self, uuid: Uuid) -> Result<Request, failure::Error> {
        let url = self
            .manifest
            .resolve(uuid)
            .ok_or_else(|| format_err!("Could not found resource {} in this registry.", uuid))?;

        let url = Url::new(url)?;
        let vfs = self.schemas.locate(url.schema())?;

        let state = Request::latch();
        let req = Request::new(state.clone());

        crate::sched::spawn(move || vfs.request(&url, state));
        Ok(req)
    }

    /// Loads file asynchronously. This method will returns a `Request` object immediatedly,
    /// its user's responsibility to store the object and frequently check it for completion.
    pub fn load_from<T: AsRef<str>>(&self, filename: T) -> Result<Request, failure::Error> {
        let filename = filename.as_ref();

        let url = self
            .shortcut
            .resolve(filename)
            .ok_or_else(|| format_err!("Could not resolve filename: {}.", filename))?;

        let uuid = self.manifest.find(&url).ok_or_else(|| {
            format_err!(
                "Could not found resource {} (resolved into {}) in this registry.",
                filename,
                url
            )
        })?;

        self.load(uuid)
    }
}
