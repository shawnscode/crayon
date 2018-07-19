use std::mem;
use std::sync::Arc;

use super::job::HeapJob;
use super::scheduler::Scheduler;
use super::unwind;

/// Spawn an asynchronous job in `Scheduler.`
pub unsafe fn spawn_in<F>(func: F, sched: &Arc<Scheduler>)
where
    F: FnOnce() + Send + 'static,
{
    // Ensure that scheduler cannot terminate until this job has executed. This
    // ref is decremented at the (*) below.
    sched.increment_terminate_count();

    let job = Box::new(HeapJob::new({
        let sched = sched.clone();
        move || {
            match unwind::halt_unwinding(func) {
                Ok(()) => {}
                Err(err) => {
                    sched.handle_panic(err);
                }
            }

            sched.terminate(); // (*) permit registry to terminate now
        }
    }));

    let abort_guard = unwind::AbortIfPanic;
    sched.inject_or_push(HeapJob::as_job_ref(job));
    mem::forget(abort_guard);
}
