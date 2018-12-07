#[cfg(not(target_arch = "wasm32"))]
pub mod dir;
#[cfg(target_arch = "wasm32")]
pub mod http;

use std::sync::Arc;

use crate::sched::prelude::LockLatch;
use crate::utils::hash::FastHashMap;

use super::request::Response;
use super::url::Url;

pub trait VFS: std::fmt::Debug + Send + Sync + 'static {
    fn request(&self, url: &Url, state: Arc<LockLatch<Response>>);
}

#[derive(Debug, Default, Clone)]
pub struct SchemaResolver {
    schemas: FastHashMap<String, Arc<VFS>>,
}

impl SchemaResolver {
    pub fn new() -> Self {
        SchemaResolver {
            schemas: FastHashMap::default(),
        }
    }

    #[inline]
    pub fn add<T1: Into<String>, T2: VFS + 'static>(&mut self, schema: T1, vfs: T2) {
        self.schemas.insert(schema.into(), Arc::new(vfs));
    }

    #[inline]
    pub fn locate<T1: AsRef<str>>(&self, schema: T1) -> Result<Arc<VFS>, failure::Error> {
        let schema = schema.as_ref();
        let vfs = self.schemas.get(schema).ok_or_else(|| {
            format_err!("The schema of url {} has not been supported yet!", schema)
        })?;

        Ok(vfs.clone())
    }
}
