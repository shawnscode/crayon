use std::sync::{Arc, Mutex, RwLock};

use utils::FastHashMap;

use super::request::{Request, RequestState};
use super::url::Url;
use super::vfs::VFS;

pub struct Worker {
    schemas: Arc<RwLock<FastHashMap<String, Arc<VFS>>>>,
}

impl Worker {
    pub fn new() -> Self {
        let schemas = Arc::new(RwLock::new(FastHashMap::default()));

        let worker = Worker { schemas: schemas };
        worker
    }

    pub fn attach<T1: Into<String>, T2: VFS + 'static>(&self, schema: T1, vfs: T2) {
        self.schemas
            .write()
            .unwrap()
            .insert(schema.into(), Arc::new(vfs));
    }

    pub fn load_from(&self, url: Url) -> Result<Request, failure::Error> {
        let schemas = self.schemas.read().unwrap();
        let vfs = schemas
            .get(url.schema())
            .ok_or_else(|| format_err!("The schema of url {} has not been supported yet!", url))?;

        let state = Arc::new(Mutex::new(RequestState::Pending));
        let req = Request::new(state.clone());

        #[cfg(target_arch = "wasm32")]
        {
            vfs.request(url, state.clone());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let vfs = vfs.clone();
            crate::sched::spawn(move || vfs.request(url, state));
        }

        Ok(req)
    }
}
