use std::collections::HashSet;

use super::window;

/// Input subsystem, responsible for converting window messages to input state
/// and internal events.
pub struct Input {
    key_downs: HashSet<window::KeyboardButton>,
    key_presses: HashSet<window::KeyboardButton>,
    mouse_downs: HashSet<window::MouseButton>,
    mouse_presses: HashSet<window::MouseButton>,

    mouse_position: (i32, i32),
    last_mouse_position: (i32, i32),
    touch_emulation: bool,
    focused: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            key_downs: HashSet::new(),
            key_presses: HashSet::new(),
            mouse_downs: HashSet::new(),
            mouse_presses: HashSet::new(),
            mouse_position: (0, 0),
            last_mouse_position: (0, 0),
            touch_emulation: false,
            focused: false,
        }
    }

    /// Reset input to initial states.
    pub fn reset(&mut self) {
        self.key_downs.clear();
        self.key_presses.clear();
        self.mouse_downs.clear();
        self.mouse_presses.clear();
    }

    /// Set touch emulation by mouse.
    pub fn set_touch_emulation(&mut self, emulation: bool) -> &Self {
        self.touch_emulation = emulation;
        self
    }


    /// Perform one frame. This will make preparations for some internal
    /// states.
    pub fn run_one_frame(&mut self) {
        self.key_presses.clear();
        self.mouse_presses.clear();
        self.last_mouse_position = self.mouse_position;
    }

    /// Handle window messages, called from application. Returns true
    /// if received closed event.
    #[doc(hidden)]
    pub fn process(&mut self, event: window::InputDeviceEvent) {
        match event {
            window::InputDeviceEvent::GainFocus => self.focused = true,
            window::InputDeviceEvent::LostFocus => self.focused = false,
            window::InputDeviceEvent::MouseMoved(x, y) => self.mouse_position = (x, y),
            window::InputDeviceEvent::MousePressed(button) => {
                if !self.mouse_downs.contains(&button) {
                    self.mouse_downs.insert(button);
                    self.mouse_presses.insert(button);
                }
            }
            window::InputDeviceEvent::MouseReleased(button) => {
                self.mouse_downs.remove(&button);
            }
            window::InputDeviceEvent::KeyboardPressed(button) => {
                if !self.key_downs.contains(&button) {
                    self.key_downs.insert(button);
                    self.key_presses.insert(button);
                }
            }
            window::InputDeviceEvent::KeyboardReleased(button) => {
                self.key_downs.remove(&button);
            }
        }
    }

    /// Returns true if we have input focus, other vice.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Checks if a key is held down.
    pub fn is_key_down(&self, button: window::KeyboardButton) -> bool {
        self.key_downs.contains(&button)
    }

    /// Checks if a key has been pressed on this frame.
    pub fn is_key_press(&self, button: window::KeyboardButton) -> bool {
        self.key_presses.contains(&button)
    }

    /// Checks if a mouse button is held down.
    pub fn is_mouse_down(&self, button: window::MouseButton) -> bool {
        self.mouse_downs.contains(&button)
    }

    /// Checks if a mouse button has been pressed on this frame.
    pub fn is_mouse_press(&self, button: window::MouseButton) -> bool {
        self.mouse_presses.contains(&button)
    }

    /// Returns the mouse position relative to the top-left hand corner of the window.
    pub fn get_mouse_position(&self) -> (i32, i32) {
        self.mouse_position
    }
}