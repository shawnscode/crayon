use std::sync::Arc;

use super::job::HeapJob;
use super::scheduler::Scheduler;
use super::scope::Scope;
use super::unwind;

pub struct SchedulerSystem {
    scheduler: Option<Arc<Scheduler>>,
}

/// The type for a panic handling closure. Note that this same closure
/// may be invoked multiple times in parallel.
pub type PanicHandler = Fn(Box<::std::any::Any + Send>) + Send + Sync;

impl SchedulerSystem {
    pub fn new(
        num: u32,
        stack_size: Option<usize>,
        panic_handler: Option<Box<PanicHandler>>,
    ) -> Self {
        SchedulerSystem {
            scheduler: Some(Scheduler::new(num, stack_size, panic_handler)),
        }
    }

    pub fn headless() -> Self {
        SchedulerSystem { scheduler: None }
    }

    pub fn terminate(&self) {
        if let Some(ref scheduler) = self.scheduler {
            scheduler.terminate_dec();
            scheduler.wait_until_terminated();
        }
    }

    // /// Blocks current thread until latch is set. Try to keep busy by popping and stealing jobs
    // /// as necessary.
    // #[inline]
    // pub fn wait_until<T>(&self, latch: &T)
    // where
    //     T: LatchWaitProbe,
    // {
    //     if let Some(ref scheduler) = self.scheduler {
    //         scheduler.wait_until(latch);
    //     } else {
    //         latch.wait();
    //     }
    // }

    /// Spawn an asynchronous job in the global `Scheduler.`
    pub fn spawn<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(ref scheduler) = self.scheduler {
            unsafe {
                // Ensure that scheduler cannot terminate until this job has executed. This
                // ref is decremented at the (*) below.
                scheduler.terminate_inc();

                let job = Box::new(HeapJob::new({
                    let sched = scheduler.clone();
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

                scheduler.inject_or_push(HeapJob::transmute(job));
            }
        } else {
            func();
        }
    }

    /// Create a "fork-join" scope `s` and invokes the closure with a
    /// reference to `s`. This closure can then spawn asynchronous tasks
    /// into `s`. Those tasks may run asynchronously with respect to the
    /// closure; they may themselves spawn additional tasks into `s`. When
    /// the closure returns, it will block until all tasks that have been
    /// spawned into `s` complete.
    pub fn scope<'s, F, R>(&self, func: F) -> R
    where
        F: for<'r> FnOnce(&'r Scope<'s>) -> R + 's + Send,
        R: Send,
    {
        unsafe {
            if let Some(ref scheduler) = self.scheduler {
                scheduler.in_worker(|worker, _| {
                    let scope = Scope::new(Some(scheduler.clone()));

                    let result = scope.execute(func);
                    scope.wait_until_completed(worker);

                    // only None if `op` panicked, and that would have been propagated.
                    result.unwrap()
                })
            } else {
                Scope::new(None).execute(func).unwrap()
            }
        }
    }
}
