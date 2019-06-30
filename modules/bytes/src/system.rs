use std::sync::{Arc, RwLock};

use crayon::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crayon::errors::Result;
use crayon::res::utils::prelude::{ResourcePool, ResourceState};
use crayon::uuid::Uuid;

use super::assets::prelude::{BytesHandle, BytesLoader};

/// The centralized management of bytes sub-system.
pub struct BytesSystem {
    lis: LifecycleListenerHandle,
    bytes: Arc<RwLock<ResourcePool<BytesHandle, BytesLoader>>>,
}

struct BytesState {
    bytes: Arc<RwLock<ResourcePool<BytesHandle, BytesLoader>>>,
}

impl LifecycleListener for BytesState {
    fn on_pre_update(&mut self) -> Result<()> {
        self.bytes.write().unwrap().advance()?;
        Ok(())
    }
}

impl Drop for BytesSystem {
    fn drop(&mut self) {
        crayon::application::detach(self.lis);
    }
}

impl BytesSystem {
    pub fn new() -> Result<Self> {
        let bytes = Arc::new(RwLock::new(ResourcePool::new(BytesLoader::new())));

        let state = BytesState {
            bytes: bytes.clone(),
        };

        Ok(BytesSystem {
            lis: crayon::application::attach(state),
            bytes: bytes,
        })
    }


    /// Creates a byte object from file asynchronously.
    #[inline]
    pub fn create_bytes_from<T: AsRef<str>>(&self, url: T) -> Result<BytesHandle> {
        self.bytes.write().unwrap().create_from(url)
    }

    /// Creates a byte object from file asynchronously.
    #[inline]
    pub fn create_bytes_from_uuid(&self, uuid: Uuid) -> Result<BytesHandle> {
        self.bytes.write().unwrap().create_from_uuid(uuid)
    }

    #[inline]
    pub fn state(&self, handle: BytesHandle) -> ResourceState {
        self.bytes.read().unwrap().state(handle)
    }
    #[inline]
    pub fn create_bytes(&self, handle: BytesHandle) -> Option<Vec<u8>>{
        self.bytes.read().unwrap().resource(handle).cloned()
    }
}
