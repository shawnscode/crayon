use std::sync::Mutex;

use uuid::Uuid;

use application::{LifecycleListener, LifecycleListenerHandle};

use super::manifest::ManfiestResolver;
use super::request::{Request, RequestQueue, Response};
use super::shortcut::ShortcutResolver;
use super::url::Url;
use super::worker::Worker;

struct Context {
    shortcut: ShortcutResolver,
    manifest: ManfiestResolver,
    worker: Worker,
    requests: Mutex<RequestQueue>,
    lifecycle: LifecycleListenerHandle,
}

static mut CTX: *const Context = 0 as *const Context;

fn ctx() -> &'static Context {
    unsafe {
        debug_assert!(
            !CTX.is_null(),
            "resource system has not been initialized properly."
        );

        &*CTX
    }
}

struct Lifecycle {}

impl LifecycleListener for Lifecycle {
    fn on_update(&mut self) {
        ctx().requests.lock().unwrap().advance();
    }
}

/// Setup the resource system.
pub unsafe fn setup() {
    debug_assert!(CTX.is_null(), "duplicated setup of resource system.");

    let mut ctx = Context {
        shortcut: ShortcutResolver::new(),
        manifest: ManfiestResolver::new(),
        worker: Worker::new(),
        requests: Mutex::new(RequestQueue::new()),
        lifecycle: crate::application::attach(Lifecycle {}),
    };

    #[cfg(not(target_arch = "wasm32"))]
    ctx.worker.attach("file", super::vfs::dir::Dir::new());
    #[cfg(target_arch = "wasm32")]
    ctx.worker.attach("http", super::vfs::http::Http::new());

    // let res = format!("file://{}/examples/resources/", env!("CARGO_MANIFEST_DIR"));
    // ctx.shortcut.add("res:", res.clone()).unwrap();

    // let manifest = Url::new(ctx.shortcut.resolve("res:.MANIFEST").unwrap()).unwrap();
    // let req = ctx.worker.load_from(manifest).unwrap();

    // match req.response().unwrap() {
    //     Ok(bytes) => {
    //         let mut cursor = std::io::Cursor::new(&bytes);
    //         ctx.manifest.add(res, &mut cursor).unwrap();
    //     }
    //     _ => {}
    // }

    CTX = Box::into_raw(Box::new(ctx));
}

/// Discard the resource system.
pub unsafe fn discard() {
    crate::application::detach(ctx().lifecycle);
    drop(Box::from_raw(CTX as *mut Context));
    CTX = 0 as *const Context;
}

/// Checks if the resource system is enabled.
#[inline]
pub fn valid() -> bool {
    unsafe { !CTX.is_null() }
}

/// Resolve shortcuts in the provided string recursively and return None if not exists.
#[inline]
pub fn resolve<T: AsRef<str>>(url: T) -> Option<String> {
    ctx().shortcut.resolve(url.as_ref())
}

/// Return the UUID of resource located at provided path, and return None if not exists.
#[inline]
pub fn find<T: AsRef<str>>(filename: T) -> Option<Uuid> {
    let ctx = ctx();
    let filename = filename.as_ref();

    ctx.shortcut
        .resolve(filename)
        .and_then(|url| ctx.manifest.find(&url))
}

/// Checks if the resource exists in this registry.
#[inline]
pub fn exists(uuid: Uuid) -> bool {
    ctx().manifest.contains(uuid)
}

/// Loads file asynchronously with response callback.
#[inline]
pub fn load_with_callback<T>(uuid: Uuid, func: T) -> Result<(), failure::Error>
where
    T: Fn(&Response) + 'static,
{
    let req = load(uuid)?;
    ctx().requests.lock().unwrap().add(req, func);
    Ok(())
}

/// Loads file asynchronously with response callback.
#[inline]
pub fn load_from_with_callback<T1, T2>(filename: T1, func: T2) -> Result<(), failure::Error>
where
    T1: AsRef<str>,
    T2: Fn(&Response) + 'static,
{
    let req = load_from(filename)?;
    ctx().requests.lock().unwrap().add(req, func);
    Ok(())
}

/// Loads file asynchronously. This method will returns a `Request` object immediatedly,
/// its user's responsibility to store the object and frequently check it for completion.
pub fn load(uuid: Uuid) -> Result<Request, failure::Error> {
    let ctx = ctx();

    let url = ctx
        .manifest
        .resolve(uuid)
        .ok_or_else(|| format_err!("Could not found resource {} in this registry.", uuid))?;

    ctx.worker.load_from(Url::new(url)?)
}

/// Loads file asynchronously. This method will returns a `Request` object immediatedly,
/// its user's responsibility to store the object and frequently check it for completion.
pub fn load_from<T: AsRef<str>>(filename: T) -> Result<Request, failure::Error> {
    let ctx = ctx();
    let filename = filename.as_ref();

    let url = ctx
        .shortcut
        .resolve(filename)
        .ok_or_else(|| format_err!("Could not resolve filename: {}.", filename))?;

    let uuid = ctx.manifest.find(&url).ok_or_else(|| {
        format_err!(
            "Could not found resource {} (resolved into {}) in this registry.",
            filename,
            url
        )
    })?;

    load(uuid)
}
