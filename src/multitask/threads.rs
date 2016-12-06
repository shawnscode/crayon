use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::collections::VecDeque;
use std::cell::Cell;

use deque;
use deque::{Worker, Stealer, Stolen};
use rand::{self, Rng};

use super::super::utility;
use super::scope;
use super::latch::*;
use super::task::{TaskRef, TaskMode, StackTask};

pub struct ThreadPool {
    infos: Vec<ThreadInfo>,
    state: Mutex<ThreadPoolState>,
    work_available: Condvar,
}

enum Work {
    None,
    Task(TaskRef),
    Terminate,
}

// public
impl ThreadPool {
    /// Creates a new thread pool. This might block the thread until all slaves ready.
    pub fn new(num_threads: usize) -> Arc<ThreadPool> {
        let (workers, stealers): (Vec<_>, Vec<_>) = (0..num_threads).map(|_| deque::new()).unzip();

        let threads = Arc::new(ThreadPool {
            infos: stealers.into_iter()
                .map(|s| ThreadInfo::new(s))
                .collect(),
            state: Mutex::new(ThreadPoolState::new()),
            work_available: Condvar::new(),
        });

        for (index, worker) in workers.into_iter().enumerate() {
            let threads = threads.clone();
            thread::spawn(move || unsafe { main_loop(worker, threads, index) });
        }

        // Waits for the worker threads to get up and running.
        for info in &threads.infos {
            info.primed.wait();
        }

        threads
    }

    /// Returns the number of current working slaves.
    #[inline]
    pub fn len(&self) -> usize {
        self.infos.len()
    }

    /// Creates a "fork-join" scope `s` and invokes the closure with a
    /// reference to `s`. This closure can the spawn asynchronous tasks
    /// into `s`. Those tasks may run asynchronously with respect to the
    /// closure; they may themselves spawn additional tasks into `s`.
    /// When the closure returns, it will block until all tasks that have
    /// been spawned into `s` complete.
    pub fn spawn<'scope, OP>(&self, op: OP)
        where OP: for<'s> FnOnce(&'s scope::Scope<'scope>) + 'scope + Send
    {
        unsafe {
            let slave = ThreadSlave::current();
            if !slave.is_null() {
                scope::run(slave, op);
            } else {
                let task = StackTask::new(|| self.spawn(op), LockLatch::new());
                self.inject(&[task.as_task_ref()]);
                task.latch.wait();
            }
        }
    }

    /// `join` simply takes two closures and potentially runs them in parallel.
    /// Using it for rucursive, divide-and-conquer problems. Its much more efficient
    /// than `spawn`, cause tasks spwaned with `Scope` must be allocated onto the heap,
    /// whereas `join` can make exclusive use of the stack.
    pub fn join<A, B, RA, RB>(&self, op_a: A, op_b: B) -> (RA, RB)
        where A: FnOnce() -> RA + Send,
              B: FnOnce() -> RB + Send,
              RA: Send,
              RB: Send
    {
        unsafe {
            let slave = ThreadSlave::current();

            if slave.is_null() {
                let task_a = StackTask::new(op_a, LockLatch::new());
                let task_b = StackTask::new(op_b, LockLatch::new());
                self.inject(&[task_a.as_task_ref(), task_b.as_task_ref()]);
                task_a.latch.wait();
                task_b.latch.wait();

                (task_a.into_result(), task_b.into_result())
            } else {
                // create virtual wrapper for task b; this all has to be
                // done here so that the stack frame can keep it all live
                // long enough
                let task_b = StackTask::new(op_b, SpinLatch::new());
                (*slave).push(task_b.as_task_ref());

                // record how many async spawns have occurred on this thread
                // before task A is executed
                let spawn_count = (*slave).current_spawn_count();

                // execute task a; hopefully b gets stolen
                let result_a;
                {
                    let guard = utility::finally(&task_b.latch, |latch| {
                        // If another thread stole our job when we panic, we must halt unwinding
                        // until that thread is finished using it.
                        if (*ThreadSlave::current()).pop().is_none() {
                            latch.spin();
                        }
                    });
                    result_a = op_a();
                    guard.forget();
                }

                // before we can try to pop b, we have to first pop off any async spawns
                // that have occurred on this thread
                (*slave).pop_spawned_jobs(spawn_count);

                // if b was not stolen, do it ourselves, else wait for the thief to finish
                let result_b;
                if (*slave).pop().is_some() {
                    // log!(PoppedJob { worker: (*worker_thread).index() });
                    result_b = task_b.run_inline(); // not stolen, let's do it!
                } else {
                    // log!(LostJob { worker: (*worker_thread).index() });
                    (*slave).steal_until(&task_b.latch); // stolen, wait for them to finish
                    result_b = task_b.into_result();
                }

                // now result_b should be initialized
                (result_a, result_b)
            }
        }
    }

    pub fn terminate(&self) {
        {
            let mut state = self.state.lock().unwrap();
            state.terminate = true;
            for task in state.injected_tasks.drain(..) {
                unsafe {
                    task.execute(TaskMode::Abort);
                }
            }
        }
        self.work_available.notify_all();
    }
}

// private
impl ThreadPool {
    fn inject(&self, injected_tasks: &[TaskRef]) {
        {
            let mut state = self.state.lock().unwrap();

            // It should not be possible for `state.terminate` to be true
            // here. It is only set to true when the user creates (and
            // drops) a `ThreadPool`; and, in that case, they cannot be
            // calling `inject()` later, since they dropped their
            // `ThreadPool`.
            assert!(!state.terminate, "inject() sees state.terminate as true");

            state.injected_tasks.extend(injected_tasks);
        }
        self.work_available.notify_all();
    }

    fn start_working(&self, _index: usize) {
        // log!(StartWorking { index: index });
        {
            let mut state = self.state.lock().unwrap();
            state.slaves_at_work += 1;
        }
        self.work_available.notify_all();
    }

    fn wait_for_work(&self, _worker: usize, was_active: bool) -> Work {
        // log!(WaitForWork {
        //     worker: _worker,
        //     was_active: was_active,
        // });

        let mut state = self.state.lock().unwrap();

        if was_active {
            state.slaves_at_work -= 1;
        }

        loop {
            // Check if we need to terminate.
            if state.terminate {
                return Work::Terminate;
            }

            // Otherwise, if anything was injected from outside,
            // return that.  Note that this gives preference to
            // injected items over stealing from others, which is a
            // bit dubious, but then so is the opposite.
            if let Some(job) = state.injected_tasks.pop_front() {
                state.slaves_at_work += 1;
                self.work_available.notify_all();
                return Work::Task(job);
            }

            // If any of the threads are running a task, we should spin
            // up, since they may generate subworkitems.
            if state.slaves_at_work > 0 {
                return Work::None;
            }

            state = self.work_available.wait(state).unwrap();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.terminate();
    }
}

struct ThreadInfo {
    // latch is set once thread has started and we are entering into
    // the main loop
    primed: LockLatch,
    stealer: Stealer<TaskRef>,
}

impl ThreadInfo {
    fn new(stealer: Stealer<TaskRef>) -> ThreadInfo {
        ThreadInfo {
            primed: LockLatch::new(),
            stealer: stealer,
        }
    }
}

struct ThreadPoolState {
    terminate: bool,
    slaves_at_work: usize,
    injected_tasks: VecDeque<TaskRef>,
}

impl ThreadPoolState {
    pub fn new() -> ThreadPoolState {
        ThreadPoolState {
            slaves_at_work: 0,
            injected_tasks: VecDeque::new(),
            terminate: false,
        }
    }
}

///
pub struct ThreadSlave {
    worker: Worker<TaskRef>,
    stealers: Vec<Stealer<TaskRef>>,

    /// A counter tracking how many calls to `Scope::spawn` occurred
    /// on the current thread; this is used by the scope code to
    /// ensure that the depth of the local deque is maintained.
    ///
    /// The actual logic here is a bit subtle. Perhaps more subtle
    /// than it has to be. The problem is this: if you have only join,
    /// then you can easily pair each push onto the deque with a pop.
    /// But when you have spawn, you push onto the deque without a
    /// corresponding pop. The `spawn_count` is used to track how many
    /// of these "unpaired pushes" have occurred.
    ///
    /// The basic pattern is that people record the spawned count
    /// before they execute a task (let's call it N). Then, if they
    /// want to pop the local tasks that this task may have spawned,
    /// they invoke `pop_spawned_jobs` with N. `pop_spawned_jobs` will
    /// pop things from the local deque and execute them until the
    /// spawn count drops to N, or the deque is empty, whichever
    /// happens first. (Either way, it resets the spawn count to N.)
    ///
    /// So e.g. join will push the right task, record the spawn count
    /// as N, run the left task, and then pop spawned jobs. Once pop
    /// spawned jobs returns, we can go ahead and try to pop the right
    /// task -- it has either been stolen, or should be on the top of the deque.
    ///
    /// Similarly, `scope` will record the spawn count and run the
    /// main task.  It can then pop the spawned jobs. At this point,
    /// until the "all done!" latch is set, it can go and steal from
    /// other people, confident in the knowledge that the local deque
    /// is empty. This is a bit subtle: basically, since all the
    /// locally spawned tasks were popped, the only way that we are
    /// not all done is if one was stolen. If one was stolen, the
    /// stuff pushed before the scope was stolen too.
    ///
    /// Finally, we have to make sure to pop spawned tasks after we
    /// steal, so as to maintain the invariant that our local deque is
    /// empty when we go to steal.
    spawn_count: Cell<usize>,

    /// A weak random number generator.
    rng: rand::XorShiftRng,
}

// This is a bit sketchy, but basically: the ThreadSlave is
// allocated on the stack of the worker on entry and stored into this
// thread local variable. So it will remain valid at least until the
// worker is fully unwound. Using an unsafe pointer avoids the need
// for a RefCell<T> etc.
thread_local! {
    static WORKER_THREAD_STATE: Cell<*mut ThreadSlave> =
        Cell::new(0 as *mut ThreadSlave)
}

impl ThreadSlave {
    /// Gets the `ThreadSlave` index for the current thread; returns
    /// NULL if this is not a worker thread. This pointer is valid
    /// anywhere on the current thread.
    #[inline]
    pub unsafe fn current() -> *mut ThreadSlave {
        WORKER_THREAD_STATE.with(|t| t.get())
    }

    /// Sets `self` as the worker thread index for the current thread.
    /// This is done during worker thread startup.
    unsafe fn set_current(&mut self) {
        WORKER_THREAD_STATE.with(|t| {
            assert!(t.get().is_null());
            t.set(self);
        });
    }

    /// Read current value of the spawn counter.
    ///
    /// See the `spawn_count` field for an extensive comment on the
    /// meaning of the spawn counter.
    #[inline]
    pub fn current_spawn_count(&self) -> usize {
        self.spawn_count.get()
    }

    /// Increment the spawn count by 1.
    ///
    /// See the `spawn_count` field for an extensive comment on the
    /// meaning of the spawn counter.
    #[inline]
    pub fn bump_spawn_count(&self) {
        self.spawn_count.set(self.spawn_count.get() + 1);
    }

    /// Pops spawned (async) jobs until our spawn count reaches
    /// `start_count` or the deque is empty. This routine is used to
    /// ensure that the local deque is "balanced".
    ///
    /// See the `spawn_count` field for an extensive comment on the
    /// meaning of the spawn counter and use of this function.
    #[inline]
    pub unsafe fn pop_spawned_jobs(&self, start_count: usize) {
        while self.spawn_count.get() != start_count {
            if let Some(job_ref) = self.pop() {
                self.spawn_count.set(self.spawn_count.get() - 1);
                job_ref.execute(TaskMode::Execute);
            } else {
                self.spawn_count.set(start_count);
                break;
            }
        }
    }

    #[inline]
    pub unsafe fn push(&self, task: TaskRef) {
        self.worker.push(task);
    }

    /// Pop `task` from top of stack, returning `false` if it has been
    /// stolen.
    #[inline]
    pub unsafe fn pop(&self) -> Option<TaskRef> {
        self.worker.pop()
    }

    /// Keep stealing tasks until the latch is set.
    #[cold]
    pub unsafe fn steal_until(&mut self, latch: &SpinLatch) {
        let spawn_count = self.spawn_count.get();

        // If another thread stole our task when we panic, we must halt unwinding
        // until that thread is finished using it.
        let guard = utility::finally(&latch, |latch| latch.spin());
        while !latch.probe() {
            if let Some(task) = self.steal_work() {
                debug_assert!(self.spawn_count.get() == spawn_count);
                task.execute(TaskMode::Execute);
                self.pop_spawned_jobs(spawn_count);
            } else {
                thread::yield_now();
            }
        }
        guard.forget();
    }

    /// Steal a single task and return it.
    unsafe fn steal_work(&mut self) -> Option<TaskRef> {
        // at no point should we try to steal unless our local deque is empty
        debug_assert!(self.pop().is_none());

        if self.stealers.is_empty() {
            return None;
        }
        let start = self.rng.next_u32() % self.stealers.len() as u32;
        let (lo, hi) = self.stealers.split_at(start as usize);
        hi.iter()
            .chain(lo)
            .filter_map(|stealer| {
                match stealer.steal() {
                    Stolen::Empty => None,
                    Stolen::Abort => None,
                    Stolen::Data(v) => Some(v),
                }
            })
            .next()
    }
}

//
pub unsafe fn main_loop(worker: Worker<TaskRef>, master: Arc<ThreadPool>, index: usize) {
    let stealers = master.infos
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != index)
        .map(|(_, ti)| ti.stealer.clone())
        .collect::<Vec<_>>();

    let mut slave = ThreadSlave {
        worker: worker,
        stealers: stealers,
        spawn_count: Cell::new(0),
        rng: rand::weak_rng(),
    };
    slave.set_current();

    // let master know we are ready to do work
    master.infos[index].primed.set();

    // Worker threads should not panic. If they do, just abort, as the
    // internal state of the threadpool is corrupted. Note that if
    // **user code** panics, we should catch that and redirect.
    // let abort_guard = unwind::AbortIfPanic;
    let mut was_active = false;
    loop {
        match master.wait_for_work(index, was_active) {
            Work::Task(injected_task) => {
                injected_task.execute(TaskMode::Execute);
                was_active = true;
                continue;
            }
            Work::Terminate => break,
            Work::None => {}
        }

        if let Some(stolen_task) = slave.steal_work() {
            // log!(StoleWork { worker: index });
            master.start_working(index);
            assert!(slave.spawn_count.get() == 0);
            stolen_task.execute(TaskMode::Execute);
            slave.pop_spawned_jobs(0);
            was_active = true;
        } else {
            was_active = false;
        }
    }

    // Normal termination, do not abort.
    // mem::forget(abort_guard);
}