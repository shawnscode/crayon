use std::io::Read;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::application::prelude::{LifecycleListener, LifecycleListenerHandle};

use super::manifest::ManfiestResolver;
use super::request::{Request, RequestQueue, Response};
use super::shortcut::ShortcutResolver;
use super::url::Url;
use super::vfs::SchemaResolver;
use super::ResourceParams;

pub struct ResourceSystem {
    shortcut: ShortcutResolver,
    schemas: SchemaResolver,
    manifest: RwLock<ManfiestResolver>,
    requests: Arc<RequestQueue>,
    lifecycle: LifecycleListenerHandle,
}

struct Lifecycle {
    requests: Arc<RequestQueue>,
}

impl LifecycleListener for Lifecycle {
    fn on_post_update(&mut self) -> Result<(), failure::Error> {
        self.requests.advance();
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

        let requests = Arc::new(RequestQueue::new());
        let sys = ResourceSystem {
            shortcut: params.shortcuts,
            schemas: params.schemas,
            manifest: RwLock::new(ManfiestResolver::new()),
            requests: requests.clone(),
            lifecycle: crate::application::attach(Lifecycle { requests }),
        };

        Ok(sys)
    }

    /// Attach a manifest to this registry.
    #[inline]
    pub fn attach<T>(&self, prefix: T, file: &mut dyn Read) -> Result<(), failure::Error>
    where
        T: AsRef<str>,
    {
        let prefix = prefix.as_ref();
        let url = self
            .shortcut
            .resolve(prefix)
            .ok_or_else(|| format_err!("Could not resolve manifest filename: {}.", prefix))?;
        self.manifest.write().unwrap().add(url, file)
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
            .and_then(|url| self.manifest.read().unwrap().find(&url))
    }

    /// Checks if the resource exists in this registry.
    #[inline]
    pub fn exists(&self, uuid: Uuid) -> bool {
        self.manifest.read().unwrap().contains(uuid)
    }

    /// Loads file asynchronously with response callback.
    #[inline]
    pub fn load_with_callback<T>(&self, uuid: Uuid, func: T) -> Result<(), failure::Error>
    where
        T: FnOnce(Response) + Send + 'static,
    {
        let req = self.load(uuid)?;
        self.requests.add(req, func);
        Ok(())
    }

    #[inline]
    pub fn load_manifest_with_callback<T1, T2>(
        &self,
        filename: T1,
        func: T2,
    ) -> Result<(), failure::Error>
    where
        T1: AsRef<str>,
        T2: FnOnce(Response) + Send + 'static,
    {
        let filename = format!("{}{}", filename.as_ref(), super::manifest::NAME);
        let url = self
            .shortcut
            .resolve(&filename)
            .ok_or_else(|| format_err!("Could not resolve filename: {}.", filename))?;
        let url = Url::new(url)?;

        let state = Request::latch();
        let req = Request::new(state.clone());
        self.requests.add(req, func);

        let vfs = self.schemas.locate(url.schema())?;
        crate::sched::spawn(move || vfs.request(&url, state));

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
        T2: FnOnce(Response) + Send + 'static,
    {
        let filename = filename.as_ref();
        let req = self.load_from(filename)?;
        self.requests.add(req, func);
        Ok(())
    }

    /// Loads file asynchronously. This method will returns a `Request` object immediatedly,
    /// its user's responsibility to store the object and frequently check it for completion.
    pub fn load(&self, uuid: Uuid) -> Result<Request, failure::Error> {
        let url =
            self.manifest.read().unwrap().resolve(uuid).ok_or_else(|| {
                format_err!("Could not found resource {} in this registry.", uuid)
            })?;

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

        let uuid = self.manifest.read().unwrap().find(&url).ok_or_else(|| {
            format_err!(
                "Could not found resource {} (resolved into {}) in this registry.",
                filename,
                url
            )
        })?;

        self.load(uuid)
    }
}
