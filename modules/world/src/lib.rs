#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;

extern crate inlinable_string;

pub mod assets;
pub mod renderable;
pub mod scene;
pub mod spatial;
pub mod tags;
pub mod utils;

pub mod prelude {
    pub use super::assets::prelude::*;
    pub use super::renderable::prelude::*;
    pub use super::scene::Scene;
    pub use super::spatial::prelude::*;
    pub use super::Entity;
}

mod system;

pub use self::inside::{discard, setup};
pub use self::system::WorldDefaultResources;

use crayon::res::utils::prelude::ResourceState;
use std::sync::Arc;

use self::assets::prelude::{Prefab, PrefabHandle};
use self::inside::ctx;

pub type Result<T> = ::std::result::Result<T, failure::Error>;

impl_handle!(Entity);

/// Creates a prefab object.
///
/// A prefab asset acts as a template from which you can create new entity instances
/// in the world. It stores a entity and its children complete with components and
/// properties internally.
#[inline]
pub fn create_prefab(prefab: Prefab) -> Result<PrefabHandle> {
    ctx().create_prefab(prefab)
}

/// Create a prefab object from file asynchronously.
///
/// A prefab asset acts as a template from which you can create new entity instances
/// in the world. It stores a entity and its children complete with components and
/// properties internally.
#[inline]
pub fn create_prefab_from<T: AsRef<str>>(url: T) -> Result<PrefabHandle> {
    ctx().create_prefab_from(url)
}

/// Return the prefab obejct if exists.
#[inline]
pub fn prefab(handle: PrefabHandle) -> Option<Arc<Prefab>> {
    ctx().prefab(handle)
}

/// Query the resource state of specified prefab.
#[inline]
pub fn prefab_state(handle: PrefabHandle) -> ResourceState {
    ctx().prefab_state(handle)
}

/// Delete a prefab object from this world.
#[inline]
pub fn delete_prefab(handle: PrefabHandle) {
    ctx().delete_prefab(handle);
}

/// Return the default resources in this world.
#[inline]
pub fn default() -> WorldDefaultResources {
    ctx().default
}

mod inside {
    use super::system::WorldSystem;

    static mut CTX: *const WorldSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static WorldSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "world system has not been initialized properly."
            );

            &*CTX
        }
    }

    /// Setup the world system.
    pub fn setup() -> Result<(), failure::Error> {
        unsafe {
            debug_assert!(CTX.is_null(), "duplicated setup of world system.");

            let ctx = WorldSystem::new()?;
            CTX = Box::into_raw(Box::new(ctx));
            Ok(())
        }
    }

    /// Discard the world system.
    pub fn discard() {
        unsafe {
            if CTX.is_null() {
                return;
            }

            drop(Box::from_raw(CTX as *mut WorldSystem));
            CTX = std::ptr::null();
        }
    }
}
