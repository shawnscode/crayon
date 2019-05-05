use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::{mem, thread};

use crossbeam_deque as deque;

use super::job::{JobRef, StackJob};
use super::latch::{CountLatch, Latch, LatchProbe, LatchWaitProbe, LockLatch};
use super::system::PanicHandler;
use super::unwind::AbortIfPanic;

pub struct Scheduler {
    terminator: CountLatch,
    watcher: Watcher,
    threads: Vec<ThreadInfo>,

    inject_stealer: deque::Stealer<JobRef>,
    injector: Mutex<deque::Worker<JobRef>>,

    panic_handler: Option<Box<PanicHandler>>,
}

impl Scheduler {
    pub fn new(
        num: u32,
        stack_size: Option<usize>,
        panic_handler: Option<Box<PanicHandler>>,
    ) -> Arc<Self> {
        let mut stealers = Vec::new();
        let mut workers = Vec::new();

        for _ in 0..num {
            let (w, s) = deque::fifo();
            workers.push(w);
            stealers.push(s);
        }

        let (w, s) = deque::fifo();
        let stealers = stealers
            .into_iter()
            .map(|v| ThreadInfo {
                stealer: v,
                primed: LockLatch::new(),
                terminated: LockLatch::new(),
            })
            .collect();

        let scheduler = Arc::new(Scheduler {
            threads: stealers,
            injector: Mutex::new(w),
            inject_stealer: s,
            panic_handler,
            terminator: CountLatch::new(),
            watcher: Watcher(Mutex::new(()), Condvar::new()),
        });

        for (i, w) in workers.drain(..).enumerate() {
            let sc = scheduler.clone();
            let mut b = thread::Builder::new();

            if let Some(stack_size) = stack_size {
                b = b.stack_size(stack_size);
            }

            b.spawn(move || unsafe { Scheduler::main_loop(sc, i, w) })
                .unwrap();
        }

        for v in &scheduler.threads {
            v.primed.wait();
        }

        scheduler
    }

    /// Push a job into the "external jobs" queue; it will be taken by whatever
    /// worker has nothing to do.
    pub fn inject(&self, job: JobRef) {
        {
            let injector = self.injector.lock().unwrap();
            injector.push(job);
        }

        self.watcher.notify_one();
    }

    // /// Push a slice of jobs into the "external jobs" queue; it will be taken by
    // /// whatever worker has nothing to do.
    // fn inject_slice(&self, jobs: &[JobRef]) {
    //     {
    //         let injector = self.injector.lock().unwrap();
    //         for &job in jobs {
    //             injector.push(job);
    //         }
    //     }
    //     self.watcher.notify_all();
    // }

    /// Push a job into the given `registry`. If we are running on a worker thread for
    /// the registry, this will push onto the deque. Else, it will inject from the
    /// outside (which is slower).
    pub fn inject_or_push(&self, job: JobRef) {
        unsafe {
            let worker_thread = WorkerThread::current();
            if worker_thread.is_null() {
                self.inject(job);
            } else {
                (*worker_thread).push(job);
                self.watcher.notify_one();
            }
        }
    }

    /// If already in a worker-thread of this registry, just execute `op`.
    /// Otherwise, inject `op` in this thread-pool. Either way, block until `op`
    /// completes and return its return value. If `op` panics, that panic will
    /// be propagated as well.  The second argument indicates `true` if injection
    /// was performed, `false` if executed directly.
    pub fn in_worker<OP, R>(&self, op: OP) -> R
    where
        OP: FnOnce(&WorkerThread, bool) -> R + Send,
        R: Send,
    {
        unsafe {
            let worker_thread = WorkerThread::current();
            if worker_thread.is_null() {
                let job = StackJob::new(
                    |_| {
                        let worker_thread = WorkerThread::current();
                        op(&*worker_thread, true)
                    },
                    LockLatch::new(),
                );

                self.inject(job.as_job_ref());

                job.latch.wait();
                job.into_result()
            } else {
                // Perfectly valid to give them a `&T`: this is the
                // current thread, so we know the data structure won't be
                // invalidated until we return.
                op(&*worker_thread, false)
            }
        }
    }

    /// Handles panic.
    pub fn handle_panic(&self, err: Box<::std::any::Any + Send>) {
        match self.panic_handler {
            Some(ref handler) => {
                // If the customizable panic handler itself panics,
                // then we abort.
                let abort_guard = AbortIfPanic;
                handler(err);
                mem::forget(abort_guard);
            }
            None => {
                // Default panic handler aborts.
                let _ = AbortIfPanic; // let this drop.
            }
        }
    }

    // pub fn wait_until<T>(&self, latch: &T)
    // where
    //     T: LatchWaitProbe,
    // {
    //     unsafe {
    //         let worker_thread = WorkerThread::current();
    //         if worker_thread.is_null() {
    //             latch.wait();
    //         } else {
    //             (*worker_thread).wait_until(latch);
    //         }
    //     }
    // }

    #[inline]
    pub fn terminate_dec(&self) {
        self.terminator.set();
    }

    #[inline]
    pub fn terminate_inc(&self) {
        self.terminator.increment();
    }

    /// Blocks current thread until all the workers finished their jobs gracefully.
    pub fn wait_until_terminated(&self) {
        let check = || {
            for v in &self.threads {
                if !v.terminated.is_set() {
                    return true;
                }
            }

            false
        };

        while check() {
            self.watcher.notify_all();
            thread::yield_now();
        }
    }

    unsafe fn main_loop(scheduler: Arc<Scheduler>, index: usize, worker: deque::Worker<JobRef>) {
        let worker_thread = WorkerThread {
            scheduler,
            index,
            worker,
            rand: XorShift64Star::new(),
        };

        WorkerThread::set_current(&worker_thread);

        worker_thread.scheduler.threads[index].primed.set(());

        worker_thread.wait_until(&worker_thread.scheduler.terminator);

        worker_thread.scheduler.threads[index].terminated.set(());
    }
}

struct Watcher(Mutex<()>, Condvar);

impl Watcher {
    #[inline]
    fn wait_timeout(&self, ms: u64) {
        let duration = ::std::time::Duration::from_millis(ms);
        let v = self.0.lock().unwrap();
        let _ = self.1.wait_timeout(v, duration);
    }

    #[inline]
    pub fn notify_one(&self) {
        self.1.notify_one()
    }

    #[inline]
    pub fn notify_all(&self) {
        self.1.notify_all()
    }
}

pub struct WorkerThread {
    scheduler: Arc<Scheduler>,
    index: usize,
    worker: deque::Worker<JobRef>,
    rand: XorShift64Star,
}

// This is a bit sketchy, but basically: the WorkerThread is allocated on the
// stack of the worker on entry and stored into this thread local variable. So
// it will remain valid at least until the worker is fully unwound. Using an
// unsafe pointer avoids the need for a RefCell<T> etc.
thread_local! {
    static WORKER_THREAD_STATE: Cell<*const WorkerThread> = Cell::new(std::ptr::null());
}

impl WorkerThread {
    /// Gets the `WorkerThread` index for the current thread; returns
    /// NULL if this is not a worker thread. This pointer is valid
    /// anywhere on the current thread.
    #[inline]
    pub fn current() -> *const WorkerThread {
        WORKER_THREAD_STATE.with(|t| t.get())
    }

    /// Sets `self` as the worker thread index for the current thread.
    /// This is done during worker thread startup.
    unsafe fn set_current(thread: *const WorkerThread) {
        WORKER_THREAD_STATE.with(|t| {
            assert!(t.get().is_null());
            t.set(thread);
        });
    }
}

impl WorkerThread {
    /// Pushs a job to `local` queue.
    #[inline]
    pub unsafe fn push(&self, job: JobRef) {
        self.worker.push(job);
    }

    pub unsafe fn wait_until<L: LatchProbe>(&self, latch: &L) {
        let abort_guard = AbortIfPanic {};
        let mut ms = 1;

        while !latch.is_set() {
            if let Some(job) = self
                .steal_local()
                .or_else(|| self.steal())
                .or_else(|| self.scheduler.inject_stealer.steal())
            {
                job.execute();
                self.scheduler.watcher.notify_all();
                ms = 1;
            } else {
                self.scheduler.watcher.wait_timeout(ms);
                ms = (ms * 2).min(48);
            }
        }

        mem::forget(abort_guard);
    }

    /// Attempts to obtain a "local" job.
    #[inline]
    unsafe fn steal_local(&self) -> Option<JobRef> {
        self.worker.pop()
    }

    /// Try to steal a single job and return it.
    unsafe fn steal(&self) -> Option<JobRef> {
        let num_threads = self.scheduler.threads.len();
        if num_threads <= 1 {
            return None;
        }

        let start = self.rand.next_usize(num_threads);
        (start..num_threads)
            .chain(0..start)
            .filter(|&i| i != self.index)
            .filter_map(|i| self.scheduler.threads[i].stealer.steal())
            .next()
    }
}

struct ThreadInfo {
    stealer: deque::Stealer<JobRef>,
    primed: LockLatch<()>,
    terminated: LockLatch<()>,
}

/// [xorshift*] is a fast pseudorandom number generator which will even tolerate
/// weak seeding, as long as it's not zero.
///
/// [xorshift*]: https://en.wikipedia.org/wiki/Xorshift#xorshift*
struct XorShift64Star {
    state: Cell<u64>,
}

impl XorShift64Star {
    fn new() -> Self {
        use crate::utils::hash;

        // Any non-zero seed will do -- this uses the hash of a global counter.
        let mut seed = 0;
        while seed == 0 {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            seed = hash::hash64(&COUNTER.fetch_add(1, Ordering::Relaxed));
        }

        XorShift64Star {
            state: Cell::new(seed),
        }
    }

    fn next(&self) -> u64 {
        let mut x = self.state.get();
        debug_assert_ne!(x, 0);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state.set(x);
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// Return a value from `0..n`.
    fn next_usize(&self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}
