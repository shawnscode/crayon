use super::latch::Latch;
use std::any::Any;
use std::cell::UnsafeCell;
use std::mem;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

enum TaskResult<T> {
    None,
    Ok(T),
    Panic(Box<Any + Send>),
}

/// A `Task` is used to advertise work for other threads that they may
/// want to steal. In accordance with time honored tradition, tasks are
/// arranged in a deque, so that thieves can take from the top of the
/// deque while the main worker manages the bottom of the deque. This
/// deque is managed by the `threads` module.
pub trait Task {
    unsafe fn execute(this: *const Self, mode: TaskMode);
}

pub enum TaskMode {
    Execute,
    Abort,
}

/// Effectively a Task trait object. Each TaskRef **must** be executed
/// exactly once, or else data may leak.
///
/// Internally, we store the task's data in a `*const ()` pointer. The
/// true type is something like `*const StackTask<...>`, but we hide
/// it. We also carry the "execute fn" from the `Task` trait.
#[derive(Copy, Clone)]
pub struct TaskRef {
    pointer: *const (),
    execute_fn: unsafe fn(*const (), mode: TaskMode),
}

unsafe impl Send for TaskRef {}
unsafe impl Sync for TaskRef {}

impl TaskRef {
    pub unsafe fn new<T>(data: *const T) -> TaskRef
        where T: Task
    {
        let fn_ptr: unsafe fn(*const T, TaskMode) = <T as Task>::execute;

        // erase types:
        let fn_ptr: unsafe fn(*const (), TaskMode) = mem::transmute(fn_ptr);
        let pointer = data as *const ();

        TaskRef {
            pointer: pointer,
            execute_fn: fn_ptr,
        }
    }

    #[inline]
    pub unsafe fn execute(&self, mode: TaskMode) {
        (self.execute_fn)(self.pointer, mode)
    }
}

/// A task that will be owned by a stack slot. This means that when it
/// executes it need not free any heap data, the cleanup occurs when
/// the stack frame is later popped.
pub struct StackTask<L: Latch, F, R> {
    pub latch: L,
    func: UnsafeCell<Option<F>>,
    result: UnsafeCell<TaskResult<R>>,
}

impl<L: Latch, F, R> StackTask<L, F, R>
    where F: FnOnce() -> R + Send
{
    pub fn new(func: F, latch: L) -> StackTask<L, F, R> {
        StackTask {
            latch: latch,
            func: UnsafeCell::new(Some(func)),
            result: UnsafeCell::new(TaskResult::None),
        }
    }

    pub unsafe fn as_task_ref(&self) -> TaskRef {
        TaskRef::new(self)
    }

    pub unsafe fn run_inline(self) -> R {
        self.func.into_inner().unwrap()()
    }

    pub unsafe fn into_result(self) -> R {
        match self.result.into_inner() {
            TaskResult::None => unreachable!(),
            TaskResult::Ok(x) => x,
            TaskResult::Panic(x) => resume_unwind(x),
        }
    }
}

impl<L: Latch, F, R> Task for StackTask<L, F, R>
    where F: FnOnce() -> R
{
    unsafe fn execute(this: *const Self, mode: TaskMode) {
        let this = &*this;
        match mode {
            TaskMode::Execute => {
                // let abort = unwind::AbortIfPanic;
                let func = (*this.func.get()).take().unwrap();
                (*this.result.get()) = match catch_unwind(AssertUnwindSafe(|| func())) {
                    Ok(x) => TaskResult::Ok(x),
                    Err(x) => TaskResult::Panic(x),
                };
                this.latch.set();
                // mem::forget(abort);
            }
            TaskMode::Abort => {
                this.latch.set();
            }
        }
    }
}

/// Represents a task stored in the heap. Used to implement
/// `scope`. Unlike `StackTask`, when executed, `HeapTask` simply
/// invokes a closure, which then triggers the appropriate logic to
/// signal that the task executed.
///
/// (Probably `StackTask` should be refactored in a similar fashion.)
pub struct HeapTask<BODY>
    where BODY: FnOnce(TaskMode)
{
    task: UnsafeCell<Option<BODY>>,
}

impl<BODY> HeapTask<BODY>
    where BODY: FnOnce(TaskMode)
{
    pub fn new(func: BODY) -> Self {
        HeapTask { task: UnsafeCell::new(Some(func)) }
    }

    /// Creates a `TaskRef` from this task -- note that this hides all
    /// lifetimes, so it is up to you to ensure that this TaskRef
    /// doesn't outlive any data that it closes over.
    pub unsafe fn as_task_ref(self: Box<Self>) -> TaskRef {
        let this: *const Self = mem::transmute(self);
        TaskRef::new(this)
    }
}

impl<BODY> Task for HeapTask<BODY>
    where BODY: FnOnce(TaskMode)
{
    unsafe fn execute(this: *const Self, mode: TaskMode) {
        let this: Box<Self> = mem::transmute(this);
        let task = (*this.task.get()).take().unwrap();
        task(mode);
    }
}
