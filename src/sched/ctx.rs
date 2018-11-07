use std::sync::Arc;

use super::job::HeapJob;
use super::latch::LatchWaitProbe;
use super::scheduler::Scheduler;
use super::scope::Scope;
use super::unwind;

struct Context {
    scheduler: Arc<Scheduler>,
}

static mut CTX: *const Context = 0 as *const Context;

fn ctx() -> &'static Context {
    unsafe {
        debug_assert!(
            !CTX.is_null(),
            "sched system has not been initialized properly."
        );

        &*CTX
    }
}

/// The type for a panic handling closure. Note that this same closure
/// may be invoked multiple times in parallel.
pub type PanicHandler = Fn(Box<::std::any::Any + Send>) + Send + Sync;

/// Setup the sched system.
pub unsafe fn setup(num: u32, stack_size: Option<usize>, panic_handler: Option<Box<PanicHandler>>) {
    debug_assert!(CTX.is_null(), "duplicated setup of sched system.");

    let ctx = Context {
        scheduler: Scheduler::new(num, stack_size, panic_handler),
    };

    CTX = Box::into_raw(Box::new(ctx));
}

/// Discard the sched system.
pub unsafe fn discard() {
    ctx().scheduler.terminate_dec();
    ctx().scheduler.wait_until_terminated();

    drop(Box::from_raw(CTX as *mut Context));
    CTX = 0 as *const Context;
}

/// Checks if the sched system is enabled.
#[inline]
pub fn valid() -> bool {
    unsafe { !CTX.is_null() }
}

/// Blocks current thread until latch is set. Try to keep busy by popping and stealing jobs
/// as necessary.
#[inline]
pub fn wait_until<T>(latch: &T)
where
    T: LatchWaitProbe,
{
    ctx().scheduler.wait_until(latch);
}

/// Spawn an asynchronous job in the global `Scheduler.`
pub fn spawn<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        let ctx = ctx();

        // Ensure that scheduler cannot terminate until this job has executed. This
        // ref is decremented at the (*) below.
        ctx.scheduler.terminate_inc();

        let job = Box::new(HeapJob::new({
            let sched = ctx.scheduler.clone();
            move || {
                match unwind::halt_unwinding(func) {
                    Ok(()) => {}
                    Err(err) => {
                        sched.handle_panic(err);
                    }
                }

                sched.terminate_dec(); // (*) permit registry to terminate now
            }
        }));

        ctx.scheduler.inject_or_push(HeapJob::as_job_ref(job));
    }
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
    ctx().scheduler.in_worker(|worker, _| unsafe {
        let scope = Scope::new(ctx().scheduler.clone());

        let result = scope.execute(func);
        scope.wait_until_completed(worker);

        // only None if `op` panicked, and that would have been propagated.
        result.unwrap()
    })
}
