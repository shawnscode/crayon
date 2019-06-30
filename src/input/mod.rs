//! Provides unified access to input devices across platforms.
//!
//! # Keyboard Inputs
//!
//! To check whether the current platform provides keyboard input, call:
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Returns true if a keyboard is attached
//! input::has_keyboard_attached();
//! ```
//!
//! Nothing bad will happen if you call the keyboard functions even if `has_keyboard_
//! attached` returns false. To check the current state of specific keys:
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Checks if a key is currently held down.
//! input::is_key_down(Key::A);
//!
//! // Checks if a key has been pressed down during the last frame.
//! input::is_key_press(Key::A);
//!
//! // Checks if a key has been released during the last frame.
//! input::is_key_repeat(Key::A);
//! ```
//!
//! A list of all key codes can be found in the `Key` enumeration. Notes
//! that the key code used here, are virtual keycode of physical keys, they don't
//! necessarily represent what's actually printed on the key cap.
//!
//! It's useful to get converted character input instead of raw key codes, to capture
//! entered text in last frame, you can call:
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Gets captured text during the last frame.
//! input::text();
//! ```
//!
//! # Mouse Inputs
//!
//! Similar to keyboard device, to find out whether the host platform provides mouse
//! input, call `has_mouse_attached`.
//!
//! To check the state of the mouse buttons, use the following functions:
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Checks if a mouse button is held down.
//! input::is_mouse_down(MouseButton::Left);
//!
//! // Checks if a mouse button has been pressed during last frame.
//! input::is_mouse_press(MouseButton::Left);
//!
//! // Checks if a mouse button has been released during last frame.
//! input::is_mouse_release(MouseButton::Left);
//! ```
//!
//! A list of all mouse buttons can be found in the `Key` enumeration. To get
//! the current mouse position and the last frame's mouse movement in pixels:
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Gets the mouse position relative to the top-left hand corner of the window.
//! input::mouse_position();
//!
//! // Gets mouse movement in pixels since last frame.
//! input::mouse_movement();
//! ```
//!
//! To get mouse wheel information:
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Gets the scroll movement of mouse in pixels, usually provided by mouse wheel.
//! input::mouse_scroll();
//! ```
//!
//! Mouse positions and movement are reported in pixel coordinates which makes it
//! difficult to derive useful movement information out of it. It might changes in
//! the future versions (dividing by the framebuffer resolution is a simple but very
//! fuzzy workaround).
//!
//! We also recognize some simple input patterns, like:
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Checks if a mouse button has been clicked during last frame.
//! input::mouse_position();
//!
//! // Checks if a mouse button has been double clicked during last frame.
//! input::is_mouse_double_click(MouseButton::Left);
//! ```
//!
//! Notes we also have APIs with `_in_points` suffix to works in logical points.
//!
//! # `TouchPad` Inputs
//!
//! The touch input functions provides access to basic touch- and multi-touch-input,
//! and is currently only implemented on mobile platforms and not for notebook
//! touchpads. You can get the touch informations by the finger index, which is
//! ordered by the first touch time.
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Checks if the `n`th finger is touched during last frame.
//! input::is_finger_touched(0);
//!
//! // Gets the position of the `n`th touched finger.
//! input::finger_position(0);
//! ```
//!
//! The touch support also addresses a few platform-agnostic gesture recognizers
//! based on low-level touch inputs.
//!
//! ```rust
//! use crayon::prelude::*;
//! application::oneshot().unwrap();
//!
//! // Gets the tap gesture.
//! match input::finger_tap() {
//!     // A tap geture is detected during last frame.
//!     GestureTap::Action { position } => { },
//!     GestureTap::None => { },
//! }
//!
//! // Gets the double tap gesture.
//! match input::finger_double_tap() {
//!     // A double tap geture is detected during last frame.
//!     GestureTap::Action { position } => { },
//!     GestureTap::None => { },
//! }
//!
//! // Gets the panning gesture.
//! match input::finger_pan() {
//!     GesturePan::Start { start_position } => { },
//!     GesturePan::Move { start_position, position, movement } => { },
//!     GesturePan::End { start_position, position } => { },
//!     GesturePan::None => { },
//! }
//! ```
//!
//! Notes we also have APIs with `_in_points` suffix to works in logical points.
//!
//! # Others Inputs
//!
//! Somethings that nice to have, but not implemented right now:
//!
//! 1. Device sensor inputs;
//! 2. Game pad inputs;
//! 3. More touch gesture like `Pinching`.

pub mod events;
pub mod keyboard;
pub mod mouse;
pub mod touchpad;

pub mod prelude {
    pub use super::events::InputEvent;
    pub use super::keyboard::{Key, KeyboardParams};
    pub use super::mouse::{MouseButton, MouseParams};
    pub use super::touchpad::{GesturePan, GestureTap, TouchPadParams};
    pub use super::InputParams;
}

mod system;

/// Maximum touches that would be tracked at sametime.
pub const MAX_TOUCHES: usize = 4;

use crate::math::prelude::Vector2;

use self::inside::{ctx, CTX};
use self::keyboard::{Key, KeyboardParams};
use self::mouse::{MouseButton, MouseParams};
use self::touchpad::{GesturePan, GestureTap, TouchPadParams};
use crate::utils::hash::FastHashSet;

/// The setup parameters of all supported input devices.
#[derive(Debug, Clone, Copy, Default)]
pub struct InputParams {
    pub touch_emulation: bool,
    pub keyboard: KeyboardParams,
    pub mouse: MouseParams,
    pub touchpad: TouchPadParams,
}

/// Checks if the resource system is enabled.
#[inline]
pub fn valid() -> bool {
    unsafe { !CTX.is_null() }
}

/// Reset input to initial states.
#[inline]
pub fn reset() {
    ctx().reset();
}

/// Returns true if a keyboard is attached
#[inline]
pub fn has_keyboard_attached() -> bool {
    ctx().has_keyboard_attached()
}

/// Checks if a key is currently held down.
#[inline]
pub fn is_key_down(key: Key) -> bool {
    ctx().is_key_down(key)
}

/// Checks if a key has been pressed down during the last frame.
#[inline]
pub fn is_key_press(key: Key) -> bool {
    ctx().is_key_press(key)
}

#[inline]
pub fn key_presses() -> FastHashSet<Key> {
    ctx().key_presses()
}

/// Checks if a key has been released during the last frame.
#[inline]
pub fn is_key_release(key: Key) -> bool {
    ctx().is_key_release(key)
}
#[inline]
pub fn key_releases() -> FastHashSet<Key>{
    ctx().key_releases()
}
/// Checks if a key has been repeated during the last frame.
#[inline]
pub fn is_key_repeat(key: Key) -> bool {
    ctx().is_key_repeat(key)
}

/// Gets captured text during the last frame.
#[inline]
pub fn text() -> String {
    ctx().text()
}

/// Returns true if a mouse is attached
#[inline]
pub fn has_mouse_attached() -> bool {
    ctx().has_mouse_attached()
}

/// Checks if a mouse buttoAn is held down.
#[inline]
pub fn is_mouse_down(button: MouseButton) -> bool {
    ctx().is_mouse_down(button)
}

/// Checks if a mouse button has been pressed during last frame.
#[inline]
pub fn is_mouse_press(button: MouseButton) -> bool {
    ctx().is_mouse_press(button)
}

/// Checks if a mouse button has been pressed during last frame.
#[inline]
pub fn mouse_presses() -> FastHashSet<MouseButton> {
    ctx().mouse_presses()
}

/// Checks if a mouse button has been released during last frame.
#[inline]
pub fn is_mouse_release(button: MouseButton) -> bool {
    ctx().is_mouse_release(button)
}

#[inline]
pub fn mouse_releases() -> FastHashSet<MouseButton> {
    ctx().mouse_releases()
}
/// Checks if a mouse button has been clicked during last frame.
#[inline]
pub fn is_mouse_click(button: MouseButton) -> bool {
    ctx().is_mouse_click(button)
}

/// Checks if a mouse button has been double clicked during last frame.
#[inline]
pub fn is_mouse_double_click(button: MouseButton) -> bool {
    ctx().is_mouse_double_click(button)
}

/// Gets the mouse position relative to the lower-left hand corner of the window.
#[inline]
pub fn mouse_position() -> Vector2<f32> {
    ctx().mouse_position()
}

/// Gets mouse movement since last frame.
#[inline]
pub fn mouse_movement() -> Vector2<f32> {
    ctx().mouse_movement()
}

/// Gets the scroll movement of mouse, usually provided by mouse wheel.
#[inline]
pub fn mouse_scroll() -> Vector2<f32> {
    ctx().mouse_scroll()
}

/// Returns true if a touchpad is attached
#[inline]
pub fn has_touchpad_attached() -> bool {
    ctx().has_touchpad_attached()
}

/// Checks if the `n`th finger is touched during last frame.
#[inline]
pub fn is_finger_touched(n: usize) -> bool {
    ctx().is_finger_touched(n)
}

/// Gets the position of the `n`th touched finger.
#[inline]
pub fn finger_position(n: usize) -> Option<Vector2<f32>> {
    ctx().finger_position(n)
}

/// Gets the tap gesture.
#[inline]
pub fn finger_tap() -> GestureTap {
    ctx().finger_tap()
}

/// Gets the double tap gesture.
#[inline]
pub fn finger_double_tap() -> GestureTap {
    ctx().finger_double_tap()
}

/// Gets the panning gesture.
#[inline]
pub fn finger_pan() -> GesturePan {
    ctx().finger_pan()
}

pub(crate) mod inside {
    use super::system::InputSystem;
    use super::InputParams;

    pub static mut CTX: *const InputSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static InputSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "input system has not been initialized properly."
            );

            &*CTX
        }
    }

    /// Setup the resource system.
    pub unsafe fn setup(params: InputParams) {
        debug_assert!(CTX.is_null(), "duplicated setup of resource system.");

        let ctx = InputSystem::new(params);
        CTX = Box::into_raw(Box::new(ctx));
    }

    /// Discard the resource system.
    pub unsafe fn discard() {
        if CTX.is_null() {
            return;
        }

        drop(Box::from_raw(CTX as *mut InputSystem));
        CTX = std::ptr::null();
    }
}
