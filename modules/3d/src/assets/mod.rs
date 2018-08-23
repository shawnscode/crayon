pub mod prefab;
pub use self::prefab::{Prefab, PrefabHandle};

pub mod prefab_loader;
pub use self::prefab_loader::PrefabLoader;

use std::sync::{Arc, RwLock};

use crayon::application::Engine;
use crayon::errors::*;
use crayon::utils::object_pool::ObjectPool;

pub struct WorldResources {
    shared: Arc<WorldResourcesShared>,
}

impl WorldResources {
    pub fn new(engine: &mut Engine) -> Self {
        let shared = Arc::new(WorldResourcesShared::new());
        let loader = PrefabLoader::new(engine.res.shared(), shared.clone());
        engine.res.register(loader);

        WorldResources { shared: shared }
    }

    pub fn shared(&self) -> Arc<WorldResourcesShared> {
        self.shared.clone()
    }
}

enum AsyncState<T> {
    Ok(T),
    NotReady,
}

pub struct WorldResourcesShared {
    prefabs: RwLock<ObjectPool<AsyncState<Arc<Prefab>>>>,
}

impl WorldResourcesShared {
    fn new() -> Self {
        WorldResourcesShared {
            prefabs: RwLock::new(ObjectPool::new()),
        }
    }

    pub(crate) fn create_prefab_async(&self) -> PrefabHandle {
        self.prefabs
            .write()
            .unwrap()
            .create(AsyncState::NotReady)
            .into()
    }

    pub(crate) fn update_prefab_async(
        &self,
        handle: PrefabHandle,
        prefab: Prefab,
    ) -> Result<Option<Prefab>> {
        prefab.validate()?;

        if let Some(v) = self.prefabs.write().unwrap().get_mut(handle) {
            *v = AsyncState::Ok(Arc::new(prefab));
            Ok(None)
        } else {
            Ok(Some(prefab))
        }
    }

    pub(crate) fn delete_prefab_async(&self, handle: PrefabHandle) -> Option<Arc<Prefab>> {
        self.prefabs
            .write()
            .unwrap()
            .free(handle)
            .and_then(|v| match v {
                AsyncState::Ok(prefab) => Some(prefab),
                _ => None,
            })
    }

    #[inline]
    pub fn prefab(&self, handle: PrefabHandle) -> Option<Arc<Prefab>> {
        if let Some(AsyncState::Ok(v)) = self.prefabs.read().unwrap().get(handle) {
            Some(v.clone())
        } else {
            None
        }
    }
}
