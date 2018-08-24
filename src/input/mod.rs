//! Provides unified access to input devices across platforms.
//!
//! # Keyboard Inputs
//!
//! To check whether the current platform provides keyboard input, call:
//!
//! ```rust
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
//! // Returns true if a keyboard is attached
//! input.has_keyboard_attached();
//! ```
//!
//! Nothing bad will happen if you call the keyboard functions even if `has_keyboard_
//! attached` returns false. To check the current state of specific keys:
//!
//! ```rust
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
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
//! A list of all key codes can be found in the `KeyboardButton` enumeration. Notes
//! that the key code used here, are virtual keycode of physical keys, they don't
//! necessarily represent what's actually printed on the key cap.
//!
//! It's useful to get converted character input instead of raw key codes, to capture
//! entered text in last frame, you can call:
//!
//! ```rust
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
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
//! ```rust
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
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
//! A list of all mouse buttons can be found in the `KeyboardButton` enumeration. To get
//! the current mouse position and the last frame's mouse movement in pixels:
//!
//! ```rust,ignore
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
//! // Gets the mouse position relative to the top-left hand corner of the window.
//! input.mouse_position();
//!
//! // Gets mouse movement in pixels since last frame.
//! input.mouse_movement();
//! ```
//!
//! To get mouse wheel information:
//!
//! ```rust
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
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
//! ```rust
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
//! // Checks if a mouse button has been clicked during last frame.
//! input.mouse_position();
//!
//! // Checks if a mouse button has been double clicked during last frame.
//! input.is_mouse_double_click(MouseButton::Left);
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
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
//! // Checks if the `n`th finger is touched during last frame.
//! input.is_finger_touched(0);
//!
//! // Gets the position of the `n`th touched finger.
//! input.finger_position(0);
//! ```
//!
//! The touch support also addresses a few platform-agnostic gesture recognizers
//! based on low-level touch inputs.
//!
//! ```rust
//! use crayon::input::prelude::*;
//! let input = InputSystem::new(InputParams::default()).shared();
//!
//! // Gets the tap gesture.
//! match input.finger_tap() {
//!     // A tap geture is detected during last frame.
//!     GestureTap::Action { position } => { },
//!     GestureTap::None => { },
//! }
//!
//! // Gets the double tap gesture.
//! match input.finger_double_tap() {
//!     // A double tap geture is detected during last frame.
//!     GestureTap::Action { position } => { },
//!     GestureTap::None => { },
//! }
//!
//! // Gets the panning gesture.
//! match input.finger_pan() {
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

pub mod keyboard;
pub mod mouse;
pub mod touchpad;

/// Maximum touches that would be tracked at sametime.
pub const MAX_TOUCHES: usize = 4;

pub mod prelude {
    pub use super::keyboard::{KeyboardButton, KeyboardParams};
    pub use super::mouse::{MouseButton, MouseParams};
    pub use super::touchpad::{GesturePan, GestureTap, TouchPadParams};
    pub use super::{InputParams, InputSystem, InputSystemShared};
}

use std::sync::{Arc, RwLock};

use application::event::{self, KeyboardButton, MouseButton};
use math;

/// The setup parameters of all supported input devices.
#[derive(Debug, Clone, Copy, Default)]
pub struct InputParams {
    pub keyboard: keyboard::KeyboardParams,
    pub mouse: mouse::MouseParams,
    pub touchpad: touchpad::TouchPadParams,
}

/// The `InputSystem` struct are used to manage all the events and corresponding
/// internal states.
pub struct InputSystem {
    touch_emulation: bool,
    touch_emulation_button: Option<MouseButton>,
    shared: Arc<InputSystemShared>,
}

impl InputSystem {
    pub fn new(setup: InputParams) -> Self {
        let shared = Arc::new(InputSystemShared::new(setup));

        InputSystem {
            shared: shared,
            touch_emulation: false,
            touch_emulation_button: None,
        }
    }

    /// Returns the multi-thread friendly parts of `InputSystem`.
    pub fn shared(&self) -> Arc<InputSystemShared> {
        self.shared.clone()
    }

    /// Reset input to initial states.
    pub fn reset(&mut self) {
        self.shared.mouse.write().unwrap().reset();
        self.shared.keyboard.write().unwrap().reset();
        self.shared.touchpad.write().unwrap().reset();
        self.touch_emulation_button = None;
    }

    /// Set touch emulation by mouse.
    pub fn set_touch_emulation(&mut self, emulation: bool) -> &Self {
        self.touch_emulation = emulation;
        self
    }

    pub(crate) fn advance(&mut self, hidpi: f32) {
        *self.shared.hidpi.write().unwrap() = hidpi;
        self.shared.mouse.write().unwrap().advance();
        self.shared.keyboard.write().unwrap().advance();
        self.shared.touchpad.write().unwrap().advance();
    }

    pub(crate) fn update_with(&mut self, v: event::InputDeviceEvent) {
        match v {
            event::InputDeviceEvent::MouseMoved { position } => {
                if self.touch_emulation_button.is_some() {
                    let touch = event::TouchEvent {
                        id: 255,
                        state: event::TouchState::Move,
                        position: self.shared.mouse.read().unwrap().position(),
                    };

                    self.shared.touchpad.write().unwrap().on_touch(touch);
                }

                self.shared.mouse.write().unwrap().on_move(position)
            }

            event::InputDeviceEvent::MousePressed { button } => {
                if self.touch_emulation {
                    self.touch_emulation_button = Some(button);

                    let touch = event::TouchEvent {
                        id: 255,
                        state: event::TouchState::Start,
                        position: self.shared.mouse.read().unwrap().position(),
                    };

                    self.shared.touchpad.write().unwrap().on_touch(touch);
                }

                self.shared.mouse.write().unwrap().on_button_pressed(button)
            }

            event::InputDeviceEvent::MouseReleased { button } => {
                if self.touch_emulation_button == Some(button) {
                    self.touch_emulation_button = None;

                    let touch = event::TouchEvent {
                        id: 255,
                        state: event::TouchState::End,
                        position: self.shared.mouse.read().unwrap().position(),
                    };

                    self.shared.touchpad.write().unwrap().on_touch(touch);
                }

                self.shared
                    .mouse
                    .write()
                    .unwrap()
                    .on_button_released(button)
            }

            event::InputDeviceEvent::MouseWheel { delta } => {
                self.shared.mouse.write().unwrap().on_wheel_scroll(delta)
            }

            event::InputDeviceEvent::KeyboardPressed { key } => {
                self.shared.keyboard.write().unwrap().on_key_pressed(key)
            }

            event::InputDeviceEvent::KeyboardReleased { key } => {
                self.shared.keyboard.write().unwrap().on_key_released(key)
            }

            event::InputDeviceEvent::ReceivedCharacter { character } => {
                self.shared.keyboard.write().unwrap().on_char(character)
            }

            event::InputDeviceEvent::Touch(touch) => {
                self.shared.touchpad.write().unwrap().on_touch(touch);
            }
        }
    }
}

/// The multi-thread friendly APIs of `InputSystem`.
pub struct InputSystemShared {
    mouse: RwLock<mouse::Mouse>,
    keyboard: RwLock<keyboard::Keyboard>,
    touchpad: RwLock<touchpad::TouchPad>,
    hidpi: RwLock<f32>,
}

impl InputSystemShared {
    fn new(setup: InputParams) -> Self {
        let kb = keyboard::Keyboard::new(setup.keyboard);
        let mice = mouse::Mouse::new(setup.mouse);
        let tp = touchpad::TouchPad::new(setup.touchpad);

        InputSystemShared {
            mouse: RwLock::new(mice),
            keyboard: RwLock::new(kb),
            touchpad: RwLock::new(tp),
            hidpi: RwLock::new(1.0),
        }
    }
}

impl InputSystemShared {
    /// Returns true if a keyboard is attached
    #[inline]
    pub fn has_keyboard_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if a key is currently held down.
    #[inline]
    pub fn is_key_down(&self, key: KeyboardButton) -> bool {
        self.keyboard.read().unwrap().is_key_down(key)
    }

    /// Checks if a key has been pressed down during the last frame.
    #[inline]
    pub fn is_key_press(&self, key: KeyboardButton) -> bool {
        self.keyboard.read().unwrap().is_key_press(key)
    }

    /// Checks if a key has been released during the last frame.
    #[inline]
    pub fn is_key_release(&self, key: KeyboardButton) -> bool {
        self.keyboard.read().unwrap().is_key_release(key)
    }

    /// Checks if a key has been repeated during the last frame.
    #[inline]
    pub fn is_key_repeat(&self, key: KeyboardButton) -> bool {
        self.keyboard.read().unwrap().is_key_repeat(key)
    }

    /// Gets captured text during the last frame.
    #[inline]
    pub fn text(&self) -> String {
        use std::iter::FromIterator;

        String::from_iter(self.keyboard.read().unwrap().captured_chars())
    }
}

impl InputSystemShared {
    /// Returns true if a mouse is attached
    #[inline]
    pub fn has_mouse_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if a mouse button is held down.
    #[inline]
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_down(button)
    }

    /// Checks if a mouse button has been pressed during last frame.
    #[inline]
    pub fn is_mouse_press(&self, button: MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_press(button)
    }

    /// Checks if a mouse button has been released during last frame.
    #[inline]
    pub fn is_mouse_release(&self, button: MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_release(button)
    }

    /// Checks if a mouse button has been clicked during last frame.
    #[inline]
    pub fn is_mouse_click(&self, button: MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_click(button)
    }

    /// Checks if a mouse button has been double clicked during last frame.
    #[inline]
    pub fn is_mouse_double_click(&self, button: MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_double_click(button)
    }

    /// Gets the mouse position in pixels relative to the lower-left hand corner of the window.
    #[inline]
    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().position() * (*self.hidpi.read().unwrap())
    }

    /// Gets the mouse position relative to the lower-left hand corner of the window.
    #[inline]
    pub fn mouse_position_in_points(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().position()
    }

    /// Gets mouse movement in pixels since last frame.
    #[inline]
    pub fn mouse_movement(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().movement() * (*self.hidpi.read().unwrap())
    }

    /// Gets mouse movement since last frame.
    #[inline]
    pub fn mouse_movement_in_points(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().movement()
    }

    /// Gets the scroll movement of mouse in pixels, usually provided by mouse wheel.
    #[inline]
    pub fn mouse_scroll(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().scroll() * (*self.hidpi.read().unwrap())
    }

    /// Gets the scroll movement of mouse, usually provided by mouse wheel.
    #[inline]
    pub fn mouse_scroll_in_points(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().scroll()
    }
}

impl InputSystemShared {
    /// Returns true if a touchpad is attached
    #[inline]
    pub fn has_touchpad_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if the `n`th finger is touched during last frame.
    #[inline]
    pub fn is_finger_touched(&self, n: usize) -> bool {
        self.touchpad.read().unwrap().is_touched(n)
    }

    /// Gets the position of the `n`th touched finger in pixels.
    #[inline]
    pub fn finger_position(&self, n: usize) -> Option<math::Vector2<f32>> {
        self.touchpad.read().unwrap().position(n)
    }

    /// Gets the position of the `n`th touched finger in pixels.
    #[inline]
    pub fn finger_position_in_points(&self, n: usize) -> Option<math::Vector2<f32>> {
        let hidpi = *self.hidpi.read().unwrap();
        self.touchpad.read().unwrap().position(n).map(|v| v * hidpi)
    }

    /// Gets the tap gesture in pixels.
    #[inline]
    pub fn finger_tap(&self) -> touchpad::GestureTap {
        self.touchpad.read().unwrap().tap()
    }

    /// Gets the tap gesture.
    #[inline]
    pub fn finger_tap_in_points(&self) -> touchpad::GestureTap {
        let hidpi = *self.hidpi.read().unwrap();
        self.touchpad.read().unwrap().tap().scale(hidpi)
    }

    /// Gets the double tap gesture in pixels.
    #[inline]
    pub fn finger_double_tap(&self) -> touchpad::GestureTap {
        self.touchpad.read().unwrap().double_tap()
    }

    /// Gets the double tap gesture.
    #[inline]
    pub fn finger_double_tap_in_points(&self) -> touchpad::GestureTap {
        let hidpi = *self.hidpi.read().unwrap();
        self.touchpad.read().unwrap().double_tap().scale(hidpi)
    }

    /// Gets the panning gesture in pixels.
    #[inline]
    pub fn finger_pan(&self) -> touchpad::GesturePan {
        self.touchpad.read().unwrap().pan()
    }

    /// Gets the panning gesture.
    #[inline]
    pub fn finger_pan_in_points(&self) -> touchpad::GesturePan {
        let hidpi = *self.hidpi.read().unwrap();
        self.touchpad.read().unwrap().pan().scale(hidpi)
    }
}
