//! The input subsystem, which is responsible for converting window messages to
//! input state and internal events.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use math;
use application::event;
use super::{keyboard, mouse, touchpad};

/// The `InputSystem` struct are used to manage all the events and corresponding
/// internal states.
pub struct InputSystem {
    touch_emulation: bool,
    shared: Arc<InputSystemShared>,
}

impl InputSystem {
    pub fn new() -> Self {
        let shared = Arc::new(InputSystemShared::new());

        InputSystem {
            shared: shared,
            touch_emulation: false,
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
    }

    /// Set touch emulation by mouse.
    pub fn set_touch_emulation(&mut self, emulation: bool) -> &Self {
        self.touch_emulation = emulation;
        self
    }

    pub(crate) fn advance(&mut self, hidpi: f32) {
        self.shared.mouse.write().unwrap().advance();
        self.shared.keyboard.write().unwrap().advance();
        self.shared.touchpad.write().unwrap().advance(hidpi);
    }

    pub(crate) fn update_with(&mut self, v: event::InputDeviceEvent) {
        match v {
            event::InputDeviceEvent::MouseMoved { position } => {
                self.shared.mouse.write().unwrap().on_move(position)
            }

            event::InputDeviceEvent::MousePressed { button } => {
                self.shared.mouse.write().unwrap().on_button_pressed(button)
            }

            event::InputDeviceEvent::MouseReleased { button } => {
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

pub struct InputSystemShared {
    mouse: RwLock<mouse::Mouse>,
    keyboard: RwLock<keyboard::Keyboard>,
    touchpad: RwLock<touchpad::TouchPad>,
}

impl InputSystemShared {
    fn new() -> Self {
        let setup = touchpad::TouchPadSetup {
            min_pan_distance: 10.0,

            tap_timeout: Duration::from_millis(750),
            max_tap_distance: 30.0,

            touch_timeout: Duration::from_millis(250),
            max_touch_distance: 20.0,
        };

        let kb = keyboard::Keyboard::new(128);
        let mice = mouse::Mouse::new();
        let tp = touchpad::TouchPad::new(setup);

        InputSystemShared {
            mouse: RwLock::new(mice),
            keyboard: RwLock::new(kb),
            touchpad: RwLock::new(tp),
        }
    }
}

impl InputSystemShared {
    /// Returns true if a keyboard is attached
    #[inline(always)]
    pub fn has_keyboard_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if a key is currently held down.
    #[inline(always)]
    pub fn is_key_down(&self, key: event::KeyboardButton) -> bool {
        self.keyboard.read().unwrap().is_key_down(key)
    }

    /// Checks if a key has been pressed down during the last frame.
    #[inline(always)]
    pub fn is_key_press(&self, key: event::KeyboardButton) -> bool {
        self.keyboard.read().unwrap().is_key_press(key)
    }

    /// Checks if a key has been released during the last frame.
    #[inline(always)]
    pub fn is_key_release(&self, key: event::KeyboardButton) -> bool {
        self.keyboard.read().unwrap().is_key_release(key)
    }

    /// Gets captured text during the last frame.
    #[inline(always)]
    pub fn text(&self) -> String {
        use std::iter::FromIterator;

        String::from_iter(self.keyboard.read().unwrap().captured_chars())
    }
}

impl InputSystemShared {
    /// Returns true if a mouse is attached
    #[inline(always)]
    pub fn has_mouse_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if a mouse button is held down.
    #[inline(always)]
    pub fn is_mouse_down(&self, button: event::MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_down(button)
    }

    /// Checks if a mouse button has been pressed during last frame.
    #[inline(always)]
    pub fn is_mouse_press(&self, button: event::MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_press(button)
    }

    /// Checks if a mouse button has been released during last frame.
    #[inline(always)]
    pub fn is_mouse_release(&self, button: event::MouseButton) -> bool {
        self.mouse.read().unwrap().is_button_release(button)
    }

    /// Returns the mouse position relative to the top-left hand corner of the window.
    #[inline(always)]
    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().position()
    }

    /// Returns mouse movement in pixels since last frame.
    #[inline(always)]
    pub fn mouse_movement(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().movement()
    }

    /// Returns the scroll movement of mouse in pixels, usually provided by mouse wheel.
    #[inline(always)]
    pub fn mouse_scroll(&self) -> math::Vector2<f32> {
        self.mouse.read().unwrap().scroll()
    }
}

impl InputSystemShared {
    /// Returns true if a touchpad is attached
    #[inline(always)]
    pub fn has_touchpad_attached(&self) -> bool {
        // TODO
        true
    }

    /// Returns true if the `n`th finger is touched.
    #[inline(always)]
    pub fn is_finger_touched(&self, n: usize) -> bool {
        self.touchpad.read().unwrap().is_touched(n)
    }

    /// Returns the position of the `n`th finger.
    #[inline(always)]
    pub fn finger_position(&self, n: usize) -> Option<math::Vector2<f32>> {
        self.touchpad.read().unwrap().position(n)
    }

    /// Gets the tap gesture.
    #[inline(always)]
    pub fn finger_tap(&self) -> touchpad::GestureTap {
        self.touchpad.read().unwrap().tap()
    }

    /// Gets the double tap gesture.
    #[inline(always)]
    pub fn finger_double_tap(&self) -> touchpad::GestureTap {
        self.touchpad.read().unwrap().double_tap()
    }

    /// Gets the panning gesture.
    #[inline(always)]
    pub fn finger_pan(&self) -> touchpad::GesturePan {
        self.touchpad.read().unwrap().pan()
    }
}