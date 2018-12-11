pub mod latch;
pub mod scope;
mod system;

mod job;
mod scheduler;
mod unwind;

pub mod prelude {
    pub use super::latch::{CountLatch, Latch, LatchProbe, LockLatch, SpinLatch};
    pub use super::system::PanicHandler;
}

use self::ins::{ctx, CTX};
use self::scope::Scope;
use self::system::{PanicHandler, SchedulerSystem};

/// Setup the sched system.
pub(crate) unsafe fn setup(
    num: u32,
    stack_size: Option<usize>,
    panic_handler: Option<Box<PanicHandler>>,
) {
    debug_assert!(CTX.is_null(), "duplicated setup of sched system.");

    CTX = Box::into_raw(Box::new(if num > 0 {
        SchedulerSystem::new(num, stack_size, panic_handler)
    } else {
        SchedulerSystem::headless()
    }));
}

/// Discard the sched system.
pub(crate) unsafe fn discard() {
    if CTX.is_null() {
        return;
    }

    drop(Box::from_raw(CTX as *mut SchedulerSystem));
    CTX = std::ptr::null();
}

pub(crate) unsafe fn terminate() {
    ctx().terminate();
}

/// Checks if the sched system is enabled.
#[inline]
pub fn valid() -> bool {
    unsafe { !CTX.is_null() }
}

// /// Blocks current thread until latch is set. Try to keep busy by popping and stealing jobs
// /// as necessary.
// #[inline]
// pub fn wait_until<T>(latch: &T)
// where
//     T: LatchWaitProbe,
// {
//     ctx().wait_until(latch);
// }

/// Spawn an asynchronous job in the global `Scheduler.`
pub fn spawn<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    ctx().spawn(func);
}

/// Create a "fork-join" scope `s` and invokes the closure with a
/// reference to `s`. This closure can then spawn asynchronous tasks
/// into `s`. Those tasks may run asynchronously with respect to the
/// closure; they may themselves spawn additional tasks into `s`. When
/// the closure returns, it will block until all tasks that have been
/// spawned into `s` complete.
pub fn scope<'s, F, R>(func: F) -> R
where
    F: for<'r> FnOnce(&'r Scope<'s>) -> R + 's + Send,
    R: Send,
{
    ctx().scope(func)
}

mod ins {
    use super::system::SchedulerSystem;

    pub static mut CTX: *const SchedulerSystem = std::ptr::null();

    pub fn ctx() -> &'static SchedulerSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "scheduler system has not been initialized properly."
            );

            &*CTX
        }
    }
}
