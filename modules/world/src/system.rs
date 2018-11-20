use std::sync::{Arc, RwLock};

use crayon::application::prelude::*;
use crayon::res::utils::prelude::*;
use crayon::video::assets::prelude::*;
use failure::Error;

use assets::prelude::*;
use assets::{mesh_builder, texture_builder};

#[derive(Debug, Clone, Copy)]
pub struct WorldDefaultResources {
    pub white: TextureHandle,
    pub cube: MeshHandle,
    pub sphere: MeshHandle,
    pub quad: MeshHandle,
}

pub struct WorldSystem {
    prefabs: Arc<RwLock<ResourcePool<PrefabHandle, PrefabLoader>>>,
    lis: LifecycleListenerHandle,

    pub default: WorldDefaultResources,
}

struct WorldState {
    prefabs: Arc<RwLock<ResourcePool<PrefabHandle, PrefabLoader>>>,
}

impl LifecycleListener for WorldState {
    fn on_pre_update(&mut self) -> Result<(), Error> {
        self.prefabs.write().unwrap().advance()?;
        Ok(())
    }
}

impl Drop for WorldSystem {
    fn drop(&mut self) {
        crayon::application::detach(self.lis);
    }
}

impl WorldSystem {
    pub fn new() -> Result<Self, Error> {
        let default = WorldDefaultResources {
            white: texture_builder::white()?,
            sphere: mesh_builder::sphere(2)?,
            cube: mesh_builder::cube()?,
            quad: mesh_builder::quad()?,
        };

        let prefabs = Arc::new(RwLock::new(ResourcePool::new(PrefabLoader::new())));

        let shared = WorldSystem {
            prefabs: prefabs.clone(),
            lis: crayon::application::attach(WorldState { prefabs }),
            default: default,
        };

        Ok(shared)
    }

    /// Create a prefab object from file asynchronously. A prefab asset acts as a template from
    /// which you can create new entity instances in the world. It stores a entity and its children
    /// complete with components and properties internally.
    #[inline]
    pub fn create_prefab_from<T: AsRef<str>>(&self, url: T) -> Result<PrefabHandle, Error> {
        let handle = self.prefabs.write().unwrap().create_from(url)?;
        Ok(handle)
    }

    /// Creates a prefab object.
    #[inline]
    pub fn create_prefab(&self, prefab: Prefab) -> Result<PrefabHandle, Error> {
        let handle = self.prefabs.write().unwrap().create(prefab)?;
        Ok(handle)
    }

    /// Return the prefab obejct if exists.
    #[inline]
    pub fn prefab(&self, handle: PrefabHandle) -> Option<Arc<Prefab>> {
        self.prefabs.read().unwrap().resource(handle).cloned()
    }

    /// Query the resource state of specified prefab.
    #[inline]
    pub fn prefab_state(&self, handle: PrefabHandle) -> ResourceState {
        self.prefabs.read().unwrap().state(handle)
    }

    /// Delete a prefab object from this world.
    #[inline]
    pub fn delete_prefab(&self, handle: PrefabHandle) {
        self.prefabs.write().unwrap().delete(handle);
    }
}
