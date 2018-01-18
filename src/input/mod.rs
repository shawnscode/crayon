//! Provides unified access to input devices across platforms.
//!
//! # Keyboard Inputs
//!
//! To check whether the current platform provides keyboard input, call:
//!
//! ```rust,ignore
//! // Returns true if a keyboard is attached
//! input.has_keyboard_attached();
//! ```
//!
//! Nothing bad will happen if you call the keyboard functions even if `has_keyboard_
//! attached` returns false. To check the current state of specific keys:
//!
//! ```rust,ignore
//! // Checks if a key is currently held down.
//! input.is_key_down(KeyboardButton::A);
//!
//! // Checks if a key has been pressed down during the last frame.
//! input.is_key_press(KeyboardButton::A);
//!
//! // Checks if a key has been released during the last frame.
//! input.is_key_repeat(KeyboardButton::A);
//! ```
//!
//! A list of all key codes can be found in the KeyboardButton enumeration. Notes
//! that the key code used here, are virtual keycode of physical keys, they don't
//! necessarily represent what's actually printed on the key cap.
//!
//! It's useful to get converted character input instead of raw key codes, to capture
//! entered text in last frame, you can call:
//!
//! ```rust,ignore
//! // Gets captured text during the last frame.
//! input.text();
//! ```
//!
//! # Mouse Inputs
//!
//! Similar to keyboard device, to find out whether the host platform provides mouse
//! input, call `has_mouse_attached`.
//!
//! To check the state of the mouse buttons, use the following functions:
//!
//! ```rust,ignore
//! // Checks if a mouse button is held down.
//! input.is_mouse_down(MouseButton::Left);
//!
//! // Checks if a mouse button has been pressed during last frame.
//! input.is_mouse_press(MouseButton::Left);
//!
//! // Checks if a mouse button has been released during last frame.
//! input.is_mouse_release(MouseButton::Left);
//! ```
//!
//! A list of all mouse buttons can be found in the KeyboardButton enumeration. To get
//! the current mouse position and the last frame's mouse movement in pixels:
//!
//! ```rust,ignore
//! // Gets the mouse position relative to the top-left hand corner of the window.
//! input.mouse_position();
//!
//! // Gets mouse movement in pixels since last frame.
//! input.mouse_movement();
//! ```
//!
//! To get mouse wheel information:
//!
//! ```rust,ignore
//! // Gets the scroll movement of mouse in pixels, usually provided by mouse wheel.
//! input.mouse_scroll();
//! ```
//!
//! Mouse positions and movement are reported in pixel coordinates which makes it
//! difficult to derive useful movement information out of it. It might changes in
//! the future versions (dividing by the framebuffer resolution is a simple but very
//! fuzzy workaround).
//!
//! We also recognize some simple input patterns, like:
//!
//! ```rust,ignore
//! // Checks if a mouse button has been clicked during last frame.
//! input.mouse_position();
//!
//! // Checks if a mouse button has been double clicked during last frame.
//! input.is_mouse_double_click();
//! ```
//!
//! # TouchPad Inputs
//!
//! The touch input functions provides access to basic touch- and multi-touch-input,
//! and is currently only implemented on mobile platforms and not for notebook
//! touchpads. You can get the touch informations by the finger index, which is
//! ordered by the first touch time.
//!
//! ```rust,ignore
//! // Checks if the `n`th finger is touched during last frame.
//! input.is_finger_touched(n);
//!
//! // Gets the position of the `n`th touched finger.
//! input.finger_position(n);
//! ```
//!
//! The touch support also addresses a few platform-agnostic gesture recognizers
//! based on low-level touch inputs.
//!
//! ```rust,ignore
//! // Gets the tap gesture.
//! match input.finger_tap() {
//!     // A tap geture is detected during last frame.
//!     GestureTap::Action { position } => { ... },
//!     GestureTap::None => { ... },
//! }
//!
//! // Gets the double tap gesture.
//! match input.finger_double_tap() {
//!     // A double tap geture is detected during last frame.
//!     GestureTap::Action { position } => { ... },
//!     GestureTap::None => { ... },
//! }
//!
//! // Gets the panning gesture.
//! match input.finger_pan() {
//!     GesturePan::Start { start_position } => { ... },
//!     GesturePan::Move { start_position, position, movement } => { ... },
//!     GesturePan::End { start_position, position } => {... },
//!     GestureTap::None => { ... },
//! }
//! ```
//!
//! # Others Inputs
//!
//! Somethings that nice to have, but not implemented right now:
//!
//! 1. Device sensor inputs;
//! 2. GamePad inputs;
//! 3. More touch gesture like `Pinching`.

mod keyboard;
mod mouse;
mod touchpad;
mod input;

pub use self::keyboard::KeyboardSetup;
pub use self::mouse::MouseSetup;
pub use self::touchpad::TouchPadSetup;
pub use self::input::{InputSetup, InputSystem, InputSystemShared};

/// Maximum touches that would be tracked at sametime.
pub const MAX_TOUCHES: usize = 4;
