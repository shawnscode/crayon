#[macro_use]
extern crate crayon;
extern crate failure;

pub mod assets;

mod system;

pub mod prelude {
    pub use assets::prelude::BytesHandle;
}

pub use self::inside::{discard, setup};

use crayon::errors::Result;
use crayon::res::prelude::ResourceState;
use crayon::uuid::Uuid;

use self::assets::prelude::BytesHandle;
use self::inside::ctx;

/// Creates a byte object from file asynchronously.
#[inline]
pub fn create_bytes_from<T: AsRef<str>>(url: T) -> Result<BytesHandle> {
    ctx().create_bytes_from(url)
}

/// Creates a byte object from file asynchronously.
#[inline]
pub fn create_bytes_from_uuid(uuid: Uuid) -> Result<BytesHandle> {
    ctx().create_bytes_from_uuid(uuid)
}

#[inline]
pub fn state(handle: BytesHandle) -> ResourceState {
    ctx().state(handle)
}
#[inline]
pub fn create_bytes(handle: BytesHandle) -> Option<Vec<u8>>{
    ctx().create_bytes(handle)
}

mod inside {
    use super::system::BytesSystem;

    static mut CTX: *const BytesSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static BytesSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "bytes system has not been initialized properly."
            );

            &*CTX
        }
    }

    /// Setup the world system.
    pub fn setup() -> Result<(), failure::Error> {
        unsafe {
            debug_assert!(CTX.is_null(), "duplicated setup of bytes system.");

            let ctx = BytesSystem::new()?;
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

            drop(Box::from_raw(CTX as *mut BytesSystem));
            CTX = std::ptr::null();
        }
    }
}
