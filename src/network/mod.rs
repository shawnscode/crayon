mod system;
mod backends;
use self::system::NetworkSystem;
use self::system::{EventListener,EventListenerHandle};
use self::ins::{ctx, CTX};
use crate::errors::Result;

/// Setup the resource system.
pub(crate) unsafe fn setup() {
    debug_assert!(CTX.is_null(), "duplicated setup of resource system.");
    let ctx = NetworkSystem::new();
    CTX = Box::into_raw(Box::new(ctx));
}

/// Discard the resource system.
pub(crate) unsafe fn discard() {
    if CTX.is_null() {
        return;
    }

    drop(Box::from_raw(CTX as *mut NetworkSystem));
    CTX = std::ptr::null();
}

/// Creates an connection
#[inline]
pub fn create_connection(params: String) -> Result<()> {
    ctx().create_connection(params)
}
/// Get receive
#[inline]
pub fn receive() -> Vec<String>{
    ctx().receive()
}
/// Get send
#[inline]
pub fn send(p:String){
    ctx().send(p);
}

mod ins {
    use super::system::NetworkSystem;

    pub static mut CTX: *const NetworkSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static NetworkSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "Network system has not been initialized properly."
            );

            &*CTX
        }
    }
}
