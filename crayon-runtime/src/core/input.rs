use std::collections::HashSet;
use glutin;

use super::event;

/// Input subsystem, responsible for converting window messages to input state
/// and internal events.
pub struct Input {
    events: glutin::EventsLoop,

    key_downs: HashSet<event::KeyboardButton>,
    key_presses: HashSet<event::KeyboardButton>,
    mouse_downs: HashSet<event::MouseButton>,
    mouse_presses: HashSet<event::MouseButton>,

    mouse_position: (i32, i32),
    last_mouse_position: (i32, i32),
    touch_emulation: bool,
    focused: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            events: glutin::EventsLoop::new(),
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

    /// Returns underlying `glutin::EventsLoop`.
    pub fn underlaying(&self) -> &glutin::EventsLoop {
        &self.events
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
    pub fn run_one_frame(&mut self, collects: &mut Vec<event::Event>) {
        self.key_presses.clear();
        self.mouse_presses.clear();
        self.last_mouse_position = self.mouse_position;

        self.events
            .poll_events(|event| {
                let event = event;
                match event {
                    glutin::Event::WindowEvent {
                        window_id: _,
                        event: v,
                    } => {
                        if let Some(parse_event) = Input::parse_window_event(v) {
                            collects.push(parse_event);
                        }
                    }

                    glutin::Event::Awakened => {
                        collects.push(event::Event::Application(event::ApplicationEvent::Awakened))
                    }

                    _ => {}
                }
            });
    }

    /// Converts `glutin::WindowEvent` into custom `Event`.
    #[doc(hidden)]
    fn parse_window_event(event: glutin::WindowEvent) -> Option<event::Event> {
        match event {
            glutin::WindowEvent::Suspended(v) => {
                Some(event::Event::Application(if v {
                                                   event::ApplicationEvent::Suspended
                                               } else {
                                                   event::ApplicationEvent::Resumed
                                               }))
            }
            glutin::WindowEvent::Closed => {
                Some(event::Event::Application(event::ApplicationEvent::Closed))
            }
            glutin::WindowEvent::Focused(v) => {
                Some(event::Event::InputDevice(if v {
                                                   event::InputDeviceEvent::GainFocus
                                               } else {
                                                   event::InputDeviceEvent::LostFocus
                                               }))
            }

            glutin::WindowEvent::MouseMoved {
                device_id: _,
                position: (x, y),
            } => {
                Some(event::Event::InputDevice(event::InputDeviceEvent::MouseMoved(x as i32,
                                                                                   y as i32)))
            }

            glutin::WindowEvent::MouseInput {
                device_id: _,
                state: glutin::ElementState::Pressed,
                button,
            } => Some(event::Event::InputDevice(event::InputDeviceEvent::MousePressed(button))),

            glutin::WindowEvent::MouseInput {
                device_id: _,
                state: glutin::ElementState::Released,
                button,
            } => Some(event::Event::InputDevice(event::InputDeviceEvent::MouseReleased(button))),

            glutin::WindowEvent::KeyboardInput {
                device_id: _,
                input: glutin::KeyboardInput {
                    scancode: _,
                    state: glutin::ElementState::Pressed,
                    virtual_keycode: Some(vkey),
                    modifiers: _,
                },
            } => Some(event::Event::InputDevice(event::InputDeviceEvent::KeyboardPressed(vkey))),

            glutin::WindowEvent::KeyboardInput {
                device_id: _,
                input: glutin::KeyboardInput {
                    scancode: _,
                    state: glutin::ElementState::Released,
                    virtual_keycode: Some(vkey),
                    modifiers: _,
                },
            } => Some(event::Event::InputDevice(event::InputDeviceEvent::KeyboardReleased(vkey))),

            _ => None,
        }
    }

    /// Handle window messages, called from application. Returns true
    /// if received closed event.
    #[doc(hidden)]
    pub fn process(&mut self, event: event::InputDeviceEvent) {
        match event {
            event::InputDeviceEvent::GainFocus => self.focused = true,
            event::InputDeviceEvent::LostFocus => self.focused = false,
            event::InputDeviceEvent::MouseMoved(x, y) => self.mouse_position = (x, y),
            event::InputDeviceEvent::MousePressed(button) => {
                if !self.mouse_downs.contains(&button) {
                    self.mouse_downs.insert(button);
                    self.mouse_presses.insert(button);
                }
            }
            event::InputDeviceEvent::MouseReleased(button) => {
                self.mouse_downs.remove(&button);
            }
            event::InputDeviceEvent::KeyboardPressed(button) => {
                if !self.key_downs.contains(&button) {
                    self.key_downs.insert(button);
                    self.key_presses.insert(button);
                }
            }
            event::InputDeviceEvent::KeyboardReleased(button) => {
                self.key_downs.remove(&button);
            }
        }
    }

    /// Returns true if we have input focus, other vice.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Checks if a key is held down.
    pub fn is_key_down(&self, button: event::KeyboardButton) -> bool {
        self.key_downs.contains(&button)
    }

    /// Checks if a key has been pressed on this frame.
    pub fn is_key_press(&self, button: event::KeyboardButton) -> bool {
        self.key_presses.contains(&button)
    }

    /// Checks if a mouse button is held down.
    pub fn is_mouse_down(&self, button: event::MouseButton) -> bool {
        self.mouse_downs.contains(&button)
    }

    /// Checks if a mouse button has been pressed on this frame.
    pub fn is_mouse_press(&self, button: event::MouseButton) -> bool {
        self.mouse_presses.contains(&button)
    }

    /// Returns the mouse position relative to the top-left hand corner of the window.
    pub fn get_mouse_position(&self) -> (i32, i32) {
        self.mouse_position
    }
}