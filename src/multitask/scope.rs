use std::any::Any;
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::panic;
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};

use super::latch::{Latch, SpinLatch};
use super::task::{TaskMode, HeapTask};
use super::threads::ThreadSlave;

pub struct Scope<'scope> {
    /// thread where `scope()` was executed (note that individual jobs
    /// may be executing on different worker threads, though they
    /// should always be within the same pool of threads)
    owner_thread: *mut ThreadSlave,

    /// number of jobs created that have not yet completed or errored
    counter: AtomicUsize,

    /// if some job panicked, the error is stored here; it will be
    /// propagated to the one who created the scope
    panic: AtomicPtr<Box<Any + Send + 'static>>,

    /// latch to set when the counter drops to zero (and hence this scope is complete)
    task_completed_latch: SpinLatch,

    /// you can think of a scope as containing a list of closures to
    /// execute, all of which outlive `'scope`
    marker: PhantomData<Box<FnOnce(&Scope<'scope>) + 'scope>>,
}

#[doc(hidden)]
pub unsafe fn run<'scope, OP>(slave: *mut ThreadSlave, op: OP)
    where OP: for<'s> FnOnce(&'s Scope<'scope>) + 'scope + Send
{
    let scope: Scope<'scope> = Scope {
        owner_thread: slave,
        counter: AtomicUsize::new(1),
        panic: AtomicPtr::new(ptr::null_mut()),
        task_completed_latch: SpinLatch::new(),
        marker: PhantomData,
    };
    let spawn_count = (*slave).current_spawn_count();
    scope.execute_forward(op);
    (*slave).pop_spawned_jobs(spawn_count);
    scope.steal_till_tasks_complete();
}

impl<'scope> Scope<'scope> {
    /// Spawns a job into the fork-join scope `self`. This job will
    /// execute sometime before the fork-join scope completes.  The
    /// job is specified as a closure, and this closure receives its
    /// own reference to `self` as argument. This can be used to
    /// inject new jobs into `self`.
    pub fn spawn<BODY>(&self, body: BODY)
        where BODY: FnOnce(&Scope<'scope>) + 'scope
    {
        unsafe {
            let old_value = self.counter.fetch_add(1, Ordering::SeqCst);
            assert!(old_value > 0); // scope can't have completed yet
            let task_ref = Box::new(HeapTask::new(move |mode| self.execute(body, mode)))
                .as_task_ref();
            let slave = ThreadSlave::current();

            // the `Scope` is not send or sync, and we only give out
            // pointers to it from within a worker thread
            debug_assert!(!ThreadSlave::current().is_null());

            let slave = &*slave;
            slave.bump_spawn_count();
            slave.push(task_ref);
        }
    }

    /// Executes `func` as a job, either aborting or executing as
    /// appropriate.
    ///
    /// Unsafe because it must be executed on a worker thread.
    unsafe fn execute<FUNC>(&self, func: FUNC, mode: TaskMode)
        where FUNC: FnOnce(&Scope<'scope>) + 'scope
    {
        match mode {
            TaskMode::Execute => self.execute_forward(func),
            TaskMode::Abort => self.set_completed(),
        }
    }

    /// Executes `func` as a job in scope. Adjusts the "job completed"
    /// counters and also catches any panic and stores it into
    /// `scope`.
    ///
    /// Unsafe because this must be executed on a worker thread.
    unsafe fn execute_forward<FUNC>(&self, func: FUNC)
        where FUNC: FnOnce(&Scope<'scope>) + 'scope
    {
        match panic::catch_unwind(panic::AssertUnwindSafe(move || func(self))) {
            Ok(()) => self.set_completed(),
            Err(err) => self.set_panic(err),
        }
    }

    unsafe fn set_panic(&self, err: Box<Any + Send + 'static>) {
        // capture the first error we see, free the rest
        let nil = ptr::null_mut();
        let mut err = Box::new(err); // box up the fat ptr
        if self.panic.compare_and_swap(nil, &mut *err, Ordering::SeqCst).is_null() {
            mem::forget(err); // ownership now transferred into self.panic
        }

        self.set_completed()
    }

    unsafe fn set_completed(&self) {
        let old_value = self.counter.fetch_sub(1, Ordering::Release);
        if old_value == 1 {
            // Important: grab the lock here to avoid a data race with
            // the `block_till_jobs_complete` code. Consider what could
            // otherwise happen:
            //
            // ```
            //    Us          Them
            //              Acquire lock
            //              Read counter: 1
            // Dec counter
            // Notify all
            //              Wait on job_completed_cvar
            // ```
            //
            // By holding the lock, we ensure that the "read counter"
            // and "wait on job_completed_cvar" occur atomically with respect to the
            // notify.
            self.task_completed_latch.set();
        }
    }

    unsafe fn steal_till_tasks_complete(&self) {
        // at this point, we have popped all tasks spawned since the scope
        // began. So either we've executed everything on this thread, or one of
        // those was stolen. If one of them was stolen, then everything below us on
        // the deque must have been stolen too, so we should just go ahead and steal.
        debug_assert!(self.task_completed_latch.probe() || (*self.owner_thread).pop().is_none());

        // wait for job counter to reach 0:
        (*self.owner_thread).steal_until(&self.task_completed_latch);

        // propagate panic, if any occurred; at this point, all
        // outstanding jobs have completed, so we can use a relaxed
        // ordering:
        let panic = self.panic.swap(ptr::null_mut(), Ordering::Relaxed);
        if !panic.is_null() {
            let value: Box<Box<Any + Send + 'static>> = mem::transmute(panic);
            panic::resume_unwind(*value);
        }
    }
}