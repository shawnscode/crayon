pub mod latch;
pub mod scope;

mod job;
mod scheduler;
mod unwind;

use std::sync::Arc;

/// The type for a panic handling closure. Note that this same closure
/// may be invoked multiple times in parallel.
type PanicHandler = Fn(Box<::std::any::Any + Send>) + Send + Sync;

pub struct ScheduleSystem {
    shared: Arc<ScheduleSystemShared>,
}

impl ScheduleSystem {
    pub fn new(
        num: u32,
        stack_size: Option<usize>,
        panic_handler: Option<Box<PanicHandler>>,
    ) -> Self {
        let shared = ScheduleSystemShared {
            scheduler: scheduler::Scheduler::new(num, stack_size, panic_handler),
        };

        ScheduleSystem {
            shared: Arc::new(shared),
        }
    }

    pub fn shared(&self) -> Arc<ScheduleSystemShared> {
        self.shared.clone()
    }

    /// Signals that the thread-pool which owns this scheduler has been dropped. Blocks current
    /// thread until all the workers finished their jobs gracefully.
    #[inline]
    pub fn terminate(&self) {
        self.shared.scheduler.terminate_dec();
        self.shared.scheduler.wait_until_terminated();
    }
}

pub struct ScheduleSystemShared {
    scheduler: Arc<scheduler::Scheduler>,
}

impl ScheduleSystemShared {
    /// Blocks current thread until latch is set. Try to keep busy by popping and stealing jobs
    /// as necessary.
    #[inline]
    pub fn wait_until<T>(&self, latch: &T)
    where
        T: latch::LatchWaitProbe,
    {
        self.scheduler.wait_until(latch);
    }

    /// Spawn an asynchronous job in `Scheduler.`
    pub fn spawn<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        unsafe {
            // Ensure that scheduler cannot terminate until this job has executed. This
            // ref is decremented at the (*) below.
            self.scheduler.terminate_inc();

            let job = Box::new(job::HeapJob::new({
                let sched = self.scheduler.clone();
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

            self.scheduler.inject_or_push(job::HeapJob::as_job_ref(job));
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
        F: for<'r> FnOnce(&'r scope::Scope<'s>) -> R + 's + Send,
        R: Send,
    {
        self.scheduler.in_worker(|worker, _| unsafe {
            let scope = scope::Scope::new(self.scheduler.clone());

            let result = scope.execute(func);
            scope.wait_until_completed(worker);

            // only None if `op` panicked, and that would have been propagated.
            result.unwrap()
        })
    }
}
