//! Represents an OpenGL context and the window or environment around it.
pub mod events;

pub mod prelude {
    pub use super::events::{Event, WindowEvent};
    pub use super::system::{EventListener, EventListenerHandle};
    pub use super::WindowParams;
}

mod backends;
mod system;

use self::ins::{ctx, CTX};
use self::system::{EventListener, EventListenerHandle, WindowSystem};

use crate::errors::*;
use crate::math::prelude::Vector2;

#[derive(Debug, Clone)]
pub struct WindowParams {
    /// Sets the title of window.
    pub title: String,
    /// Sets the size in *points* of the client area of the window.
    pub size: Vector2<u32>,
    /// Sets the multisampling level to request. A value of 0 indicates that
    /// multisampling must not be enabled.
    pub multisample: u16,
    /// Specifies whether should we have vsync.
    pub vsync: bool,
}

impl Default for WindowParams {
    fn default() -> Self {
        WindowParams {
            title: "Window".to_owned(),
            size: Vector2::new(640, 320),
            multisample: 2,
            vsync: false,
        }
    }
}

/// Setup the window system.
pub(crate) unsafe fn setup(params: WindowParams) -> Result<()> {
    debug_assert!(CTX.is_null(), "duplicated setup of window system.");

    let ctx = WindowSystem::from(params)?;
    CTX = Box::into_raw(Box::new(ctx));
    Ok(())
}

pub(crate) unsafe fn headless() {
    debug_assert!(CTX.is_null(), "duplicated setup of window system.");

    let ctx = WindowSystem::headless();
    CTX = Box::into_raw(Box::new(ctx));
}

/// Resize the GL context.
#[inline]
pub(crate) fn resize(dimensions: Vector2<u32>) {
    ctx().resize(dimensions);
}

/// Discard the window system.
pub(crate) unsafe fn discard() {
    if CTX.is_null() {
        return;
    }

    drop(Box::from_raw(CTX as *mut WindowSystem));
    CTX = std::ptr::null();
}

/// Adds a event listener.
pub fn attach<T: EventListener + 'static>(lis: T) -> EventListenerHandle {
    ctx().add_event_listener(lis)
}

/// Removes a event listener from window.
pub fn detach(handle: EventListenerHandle) {
    ctx().remove_event_listener(handle)
}

/// Shows the window if it was hidden.
///
/// # Platform-specific
///
/// Has no effect on mobile platform.
#[inline]
pub fn show() {
    ctx().show();
}

/// Hides the window if it was visible.
///
/// # Platform-specific
///
/// Has no effect on mobile platform.
#[inline]
pub fn hide() {
    ctx().hide();
}

/// Set the context as the active context in this thread.
#[inline]
pub fn make_current() -> Result<()> {
    ctx().make_current()
}

/// Returns true if this context is the current one in this thread.
#[inline]
pub fn is_current() -> bool {
    ctx().is_current()
}

/// Returns the position of the lower-left hand corner of the window relative to the lower-left
/// hand corner of the desktop. Note that the lower-left hand corner of the desktop is not
/// necessarily the same as the screen. If the user uses a desktop with multiple monitors,
/// the lower-left hand corner of the desktop is the lower-left hand corner of the monitor at
/// the lower-left of the desktop.
///
/// The coordinates can be negative if the lower-left hand corner of the window is outside of
/// the visible screen region.
#[inline]
pub fn position() -> Vector2<i32> {
    ctx().position()
}

/// Returns the size in *points* of the client area of the window.
///
/// The client area is the content of the window, excluding the title bar and borders. These are
/// the size of the frame buffer.
#[inline]
pub fn dimensions() -> Vector2<u32> {
    ctx().dimensions()
}

/// Returns the ratio between the backing framebuffer resolution and the window size in
/// screen pixels. This is typically one for a normal display and two for a retina display.
#[inline]
pub fn device_pixel_ratio() -> f32 {
    ctx().device_pixel_ratio()
}

mod ins {
    use super::system::WindowSystem;

    pub static mut CTX: *const WindowSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static WindowSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "window system has not been initialized properly."
            );

            &*CTX
        }
    }
}
