use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::{mem, ptr};

use super::job::HeapJob;
use super::latch::{CountLatch, Latch};
use super::scheduler::{Scheduler, WorkerThread};
use super::unwind;

/// Represents a fork-join scope which can be used to spawn any number of tasks.
pub struct Scope<'s> {
    scheduler: Option<Arc<Scheduler>>,
    latch: CountLatch,
    marker: PhantomData<Box<FnOnce(&Scope<'s>) + Send + Sync + 's>>,
    /// if some job panicked, the error is stored here; it will be
    /// propagated to the one who created the scope
    panic: AtomicPtr<Box<Any + Send + 'static>>,
}

impl<'s> Scope<'s> {
    pub fn new(scheduler: Option<Arc<Scheduler>>) -> Self {
        Scope {
            scheduler,
            latch: CountLatch::new(),
            marker: PhantomData::default(),
            panic: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Spawns a job into the fork-join scope `self`. This job will execute sometime before
    /// the fork-join scope completes.  The job is specified as a closure, and this closure
    /// receives its own reference to `self` as argument. This can be used to inject new jobs
    /// into `self`.
    pub fn spawn<F>(&self, func: F)
    where
        F: FnOnce(&Scope<'s>) + Send + 's,
    {
        unsafe {
            if let Some(ref scheduler) = self.scheduler {
                self.latch.increment();

                let job = Box::new(HeapJob::new(move || {
                    let _v = self.execute(func);
                }))
                .transmute();

                // Since `Scope` implements `Sync`, we can't be sure that we're still in a thread
                // of this pool, so we can't just push to the local worker thread.
                scheduler.inject_or_push(job);
            } else {
                func(self);
            }
        }
    }

    /// Executes `func` as a job in scope. Adjusts the "job completed" counters and
    /// also catches any panic and stores it into `scope`.
    pub(crate) unsafe fn execute<F, R>(&self, func: F) -> Option<R>
    where
        F: FnOnce(&Scope<'s>) -> R + 's,
    {
        match unwind::halt_unwinding(move || func(self)) {
            Ok(r) => {
                self.latch.set();
                Some(r)
            }
            Err(err) => {
                // capture the first error we see, free the rest
                let nil = ptr::null_mut();
                let mut err = Box::new(err); // box up the fat ptr
                if self
                    .panic
                    .compare_exchange(nil, &mut *err, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    mem::forget(err); // ownership now transferred into self.panic
                }

                self.latch.set();
                None
            }
        }
    }

    pub(crate) unsafe fn wait_until_completed(&self, worker: &WorkerThread) {
        // wait for job counter to reach 0:
        worker.wait_until(&self.latch);

        // propagate panic, if any occurred; at this point, all outstanding jobs have completed,
        // so we can use a relaxed ordering:
        let panic = self.panic.swap(ptr::null_mut(), Ordering::Relaxed);
        if !panic.is_null() {
            let value: Box<Box<Any + Send + 'static>> = mem::transmute(panic);
            unwind::resume_unwinding(*value);
        }
    }
}
