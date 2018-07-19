use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::{Arc, Condvar, Mutex, Once, ONCE_INIT};
use std::{mem, thread};

use crossbeam_deque as deque;

use super::job::{JobRef, StackJob};
use super::latch::{CountLatch, Latch, LockLatch};
use super::unwind::AbortIfPanic;
use super::PanicHandler;

static mut SCHEDULER: Option<&'static Arc<Scheduler>> = None;

pub unsafe fn init(num: u32, stack_size: usize, panic_handler: Option<Box<PanicHandler>>) {
    assert!(
        SCHEDULER.is_none(),
        "Scheduler has been initialized already."
    );

    SCHEDULER = Some(leak(Scheduler::new(num, stack_size, panic_handler)));
}

pub fn current() -> &'static Arc<Scheduler> {
    unsafe { SCHEDULER.expect("Scheduler has not been initialized.") }
}

pub struct Scheduler {
    threads: Vec<ThreadInfo>,

    inject_stealer: deque::Stealer<JobRef>,
    injector: Mutex<deque::Worker<JobRef>>,

    panic_handler: Option<Box<PanicHandler>>,
    terminator: CountLatch,
    signal: Signal,
}

impl Scheduler {
    pub fn new(num: u32, stack_size: usize, panic_handler: Option<Box<PanicHandler>>) -> Arc<Self> {
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
            panic_handler: panic_handler,
            terminator: CountLatch::new(),
            signal: Signal(Mutex::new(()), Condvar::new()),
        });

        for (i, w) in workers.drain(..).enumerate() {
            let sc = scheduler.clone();
            thread::Builder::new()
                .stack_size(stack_size)
                .spawn(move || unsafe { Scheduler::main_loop(sc, i, w) })
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

        self.signal.notify_one();
    }

    /// Push a slice of jobs into the "external jobs" queue; it will be taken by
    /// whatever worker has nothing to do.
    pub fn inject_slice(&self, jobs: &[JobRef]) {
        {
            let injector = self.injector.lock().unwrap();
            for &job in jobs {
                injector.push(job);
            }
        }

        self.signal.notify_all();
    }

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
                self.signal.notify_one();
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

    /// Signals that the thread-pool which owns this scheduler has been dropped.
    /// The worker threads will gradually terminate, once any extant work is
    /// completed.
    pub fn terminate(&self) {
        self.terminator.set();
    }

    pub fn increment_terminate_count(&self) {
        self.terminator.increment();
    }

    pub fn wait_until_terminated(&self) {
        self.signal.notify_all();

        for v in &self.threads {
            v.terminated.wait();
        }
    }

    unsafe fn main_loop(scheduler: Arc<Scheduler>, index: usize, worker: deque::Worker<JobRef>) {
        let worker_thread = WorkerThread {
            scheduler: scheduler,
            index: index,
            worker: worker,
            rand: XorShift64Star::new(),
        };

        WorkerThread::set_current(&worker_thread);

        worker_thread.scheduler.threads[index].primed.set();

        worker_thread.wait_until(&worker_thread.scheduler.terminator);

        worker_thread.scheduler.threads[index].terminated.set();
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
    static WORKER_THREAD_STATE: Cell<*const WorkerThread> =
        Cell::new(0 as *const WorkerThread)
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
    /// Our index amongst the worker threads (ranges from `0..self.num_threads()`).
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.worker.is_empty()
    }

    /// Pushs a job to `local` queue.
    #[inline]
    pub unsafe fn push(&self, job: JobRef) {
        self.worker.push(job);
    }

    /// Attempts to obtain a "local" job.
    #[inline]
    pub unsafe fn steal_local(&self) -> Option<JobRef> {
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

    unsafe fn wait_until<L: Latch>(&self, latch: &L) {
        let abort_guard = AbortIfPanic {};

        while !latch.is_set() {
            if let Some(job) = self.steal_local()
                .or_else(|| self.steal())
                .or_else(|| self.scheduler.inject_stealer.steal())
            {
                job.execute();
                self.scheduler.signal.notify_all();
            } else {
                self.scheduler.signal.wait();
            }
        }

        mem::forget(abort_guard);
    }
}

struct Signal(Mutex<()>, Condvar);

impl Signal {
    #[inline]
    fn wait(&self) {
        let guard = self.0.lock().unwrap();
        let _ = self.1.wait(guard).unwrap();
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

struct ThreadInfo {
    stealer: deque::Stealer<JobRef>,
    primed: LockLatch,
    terminated: LockLatch,
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
        // Any non-zero seed will do -- this uses the hash of a global counter.
        let mut seed = 0;
        while seed == 0 {
            let mut hasher = DefaultHasher::new();
            static COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
            hasher.write_usize(COUNTER.fetch_add(1, Ordering::Relaxed));
            seed = hasher.finish();
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

fn leak<T>(v: T) -> &'static T {
    unsafe {
        let b = Box::new(v);
        let p: *const T = &*b;
        mem::forget(b); // leak our reference, so that `b` is never freed
        &*p
    }
}
