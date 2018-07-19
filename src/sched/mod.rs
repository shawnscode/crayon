pub mod latch;

mod job;
mod scheduler;
mod spawn;
mod unwind;

/// The type for a panic handling closure. Note that this same closure
/// may be invoked multiple times in parallel.
type PanicHandler = Fn(Box<::std::any::Any + Send>) + Send + Sync;

pub fn init(num: u32, stack_size: usize, panic_handler: Option<Box<PanicHandler>>) {
    unsafe {
        scheduler::init(num, stack_size, panic_handler);
    }
}

pub fn spawn<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        spawn::spawn_in(func, scheduler::current());
    }
}
