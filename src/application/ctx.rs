use std::sync::Mutex;

use utils::ObjectPool;

struct Context {
    lifecycles: Mutex<ObjectPool<LifecycleListenerHandle, Box<LifecycleListener>>>,
}

static mut CTX: *const Context = 0 as *const Context;

fn ctx() -> &'static Context {
    unsafe {
        debug_assert!(
            !CTX.is_null(),
            "core system has not been initialized properly."
        );

        &*CTX
    }
}

impl_handle!(LifecycleListenerHandle);

pub trait LifecycleListener {
    fn on_update(&mut self) {}
    fn on_render(&mut self) {}
    fn on_post_update(&mut self) {}
    fn on_exit(&mut self) {}
}

/// Setup the core system.
pub unsafe fn setup() {
    debug_assert!(CTX.is_null(), "duplicated setup of core system.");

    let ctx = Context {
        lifecycles: Mutex::new(ObjectPool::new()),
    };

    CTX = Box::into_raw(Box::new(ctx))
}

/// Discard the core system.
pub unsafe fn discard() {
    foreach(|v| v.on_exit());
    drop(Box::from_raw(CTX as *mut Context));
    CTX = 0 as *const Context;
}

#[inline]
pub fn valid() -> bool {
    unsafe { !CTX.is_null() }
}

#[inline]
pub fn attach<T>(lis: T) -> LifecycleListenerHandle
where
    T: LifecycleListener + 'static,
{
    ctx().lifecycles.lock().unwrap().create(Box::new(lis))
}

#[inline]
pub fn detach(handle: LifecycleListenerHandle) {
    ctx().lifecycles.lock().unwrap().free(handle);
}

#[inline]
pub fn foreach<T: Fn(&mut dyn LifecycleListener)>(func: T) {
    let mut lifecycles = ctx().lifecycles.lock().unwrap();
    for v in lifecycles.values_mut() {
        func(v.as_mut());
    }
}
