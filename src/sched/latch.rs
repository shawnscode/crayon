use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

/// We define various kinds of latches, which are all a primitive signaling
/// mechanism. A latch starts as false. Eventually someone calls `set()` and
/// it becomes true. You can test if it has been set by calling `is_set()`.
pub trait Latch {
    /// Set the latch, signalling others.
    fn set(&self);
    /// Test if the latch is set.
    fn is_set(&self) -> bool;
}

/// Spin latches are the simplest, most efficient kind, but they do not support
/// a `wait()` operation. They just have a boolean flag that becomes true when
/// `set()` is called.
pub struct SpinLatch {
    b: AtomicBool,
}

impl SpinLatch {
    #[inline]
    pub fn new() -> SpinLatch {
        SpinLatch {
            b: AtomicBool::new(false),
        }
    }
}

impl Latch for SpinLatch {
    #[inline]
    fn set(&self) {
        self.b.store(true, Ordering::SeqCst);
    }

    #[inline]
    fn is_set(&self) -> bool {
        self.b.load(Ordering::SeqCst)
    }
}

/// A Latch starts as false and eventually becomes true. You can block until
/// it becomes true.
pub struct LockLatch {
    m: Mutex<bool>,
    v: Condvar,
}

impl LockLatch {
    #[inline]
    pub fn new() -> LockLatch {
        LockLatch {
            m: Mutex::new(false),
            v: Condvar::new(),
        }
    }

    /// Block until latch is set.
    pub fn wait(&self) {
        let mut guard = self.m.lock().unwrap();
        while !*guard {
            guard = self.v.wait(guard).unwrap();
        }
    }
}

impl Latch for LockLatch {
    #[inline]
    fn set(&self) {
        let mut guard = self.m.lock().unwrap();
        *guard = true;
        self.v.notify_all();
    }

    #[inline]
    fn is_set(&self) -> bool {
        // Not particularly efficient, but we don't really use this operation
        let guard = self.m.lock().unwrap();
        *guard
    }
}

/// Counting latches are used to implement scopes. They track a counter. Unlike
/// other latches, calling `set()` does not necessarily make the latch be
/// considered `set()`; instead, it just decrements the counter. The latch is
/// only "set" (in the sense that`is_set()` returns true) once the counter reaches zero.
#[derive(Debug)]
pub struct CountLatch {
    counter: AtomicUsize,
}

impl CountLatch {
    #[inline]
    pub fn new() -> CountLatch {
        CountLatch {
            counter: AtomicUsize::new(1),
        }
    }

    #[inline]
    pub fn increment(&self) {
        debug_assert!(!self.is_set());
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

impl Latch for CountLatch {
    #[inline]
    fn is_set(&self) -> bool {
        // Need to acquire any memory reads before latch was set:
        self.counter.load(Ordering::SeqCst) == 0
    }
    /// Set the latch to true, releasing all threads who are waiting.
    #[inline]
    fn set(&self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}
