use std::sync::Arc;

use crayon::application::prelude::*;
use crayon::errors::*;
use crayon::res::prelude::*;

use assets::mesh_builder::WorldBuiltinMeshes;
use assets::prefab::{Prefab, PrefabHandle};
use assets::prefab_loader::PrefabLoader;
use assets::texture_builder::WorldBuiltinTextures;

pub struct WorldResources {
    shared: Arc<WorldResourcesShared>,
}

impl WorldResources {
    pub fn new(engine: &mut Engine) -> Result<Self> {
        let shared = Arc::new(WorldResourcesShared::new(engine.context())?);
        Ok(WorldResources { shared: shared })
    }

    pub fn shared(&self) -> Arc<WorldResourcesShared> {
        self.shared.clone()
    }
}

pub type PrefabRegistry = Registry<PrefabHandle, PrefabLoader>;

pub struct WorldResourcesShared {
    prefabs: PrefabRegistry,

    pub meshes: WorldBuiltinMeshes,
    pub textures: WorldBuiltinTextures,
}

impl WorldResourcesShared {
    fn new(ctx: &Context) -> Result<Self> {
        let register = PrefabLoader::new(ctx);

        let shared = WorldResourcesShared {
            prefabs: PrefabRegistry::new(ctx.res.clone(), register),
            meshes: WorldBuiltinMeshes::new(ctx)?,
            textures: WorldBuiltinTextures::new(ctx)?,
        };

        Ok(shared)
    }

    #[inline]
    pub fn create_prefab_from<'a, T>(&'a self, location: T) -> Result<PrefabHandle>
    where
        T: Into<Location<'a>>,
    {
        let handle = self.prefabs.create_from(location)?;
        Ok(handle)
    }

    #[inline]
    pub fn prefab(&self, handle: PrefabHandle) -> Option<Arc<Prefab>> {
        self.prefabs
            .wait_until(handle)
            .ok()
            .and_then(|_| self.prefabs.get(handle, |v| v.clone()))
    }

    #[inline]
    pub fn delete_prefab(&self, handle: PrefabHandle) {
        self.prefabs.delete(handle);
    }
}
