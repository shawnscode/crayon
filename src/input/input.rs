//! The input subsystem, which is responsible for converting window messages to
//! input state and internal events.

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use glutin;

use super::event;

/// The `InputSystem` struct are used to manage all the events and corresponding
/// internal states.
pub struct InputSystem {
    touch_emulation: bool,
    events: glutin::EventsLoop,
    shared: Arc<InputSystemShared>,
}

impl InputSystem {
    pub fn new() -> Self {
        let data = Arc::new(RwLock::new(InputSystemData::new()));
        let shared = Arc::new(InputSystemShared::new(data));

        InputSystem {
            events: glutin::EventsLoop::new(),
            shared: shared,
            touch_emulation: false,
        }
    }

    /// Returns underlying `glutin::EventsLoop`.
    pub fn underlaying(&self) -> &glutin::EventsLoop {
        &self.events
    }

    /// Returns the multi-thread friendly parts of `InputSystem`.
    pub fn shared(&self) -> Arc<InputSystemShared> {
        self.shared.clone()
    }

    /// Reset input to initial states.
    pub fn reset(&mut self) {
        self.shared.data.write().unwrap().reset();
    }

    /// Set touch emulation by mouse.
    pub fn set_touch_emulation(&mut self, emulation: bool) -> &Self {
        self.touch_emulation = emulation;
        self
    }

    /// Perform one frame. This will make preparations for some internal
    /// states.
    pub fn run_one_frame(&mut self, collects: &mut Vec<event::Event>) {
        let mut data = self.shared.data.write().unwrap();

        data.key_presses.clear();
        data.mouse_presses.clear();

        self.events
            .poll_events(|event| {
                let event = event;
                match event {
                    glutin::Event::WindowEvent {
                        window_id: _,
                        event: v,
                    } => {
                        if let Some(parse_event) = Self::parse_window_event(v, &mut data) {
                            collects.push(parse_event);
                        }
                    }

                    glutin::Event::Awakened => {
                        collects.push(event::Event::Application(event::ApplicationEvent::Awakened))
                    }

                    glutin::Event::Suspended(v) => {
                        if v {
                            collects.push(event::Event::Application(event::ApplicationEvent::Suspended));
                        } else {
                            collects.push(event::Event::Application(event::ApplicationEvent::Resumed));
                        }
                    }

                    _ => {}
                }
            });
    }

    /// Converts `glutin::WindowEvent` into custom `Event`.
    #[doc(hidden)]
    fn parse_window_event(event: glutin::WindowEvent,
                          data: &mut InputSystemData)
                          -> Option<event::Event> {
        match event {
            glutin::WindowEvent::Closed => {
                Some(event::Event::Application(event::ApplicationEvent::Closed))
            }
            glutin::WindowEvent::Focused(v) => {
                Some(event::Event::Application(if v {
                                                   event::ApplicationEvent::GainFocus
                                               } else {
                                                   event::ApplicationEvent::LostFocus
                                               }))
            }

            glutin::WindowEvent::MouseMoved {
                device_id: _,
                position: (x, y),
            } => {
                data.mouse_position = (x as i32, y as i32);
                None
            }

            glutin::WindowEvent::MouseInput {
                device_id: _,
                state: glutin::ElementState::Pressed,
                button,
            } => {
                if !data.mouse_downs.contains(&button) {
                    data.mouse_downs.insert(button);
                    data.mouse_presses.insert(button);
                }
                None
            }

            glutin::WindowEvent::MouseInput {
                device_id: _,
                state: glutin::ElementState::Released,
                button,
            } => {
                data.mouse_downs.remove(&button);
                None
            }

            glutin::WindowEvent::KeyboardInput {
                device_id: _,
                input: glutin::KeyboardInput {
                    scancode: _,
                    state: glutin::ElementState::Pressed,
                    virtual_keycode: Some(vkey),
                    modifiers: _,
                },
            } => {
                if !data.key_downs.contains(&vkey) {
                    data.key_downs.insert(vkey);
                    data.key_presses.insert(vkey);
                }
                None
            }

            glutin::WindowEvent::KeyboardInput {
                device_id: _,
                input: glutin::KeyboardInput {
                    scancode: _,
                    state: glutin::ElementState::Released,
                    virtual_keycode: Some(vkey),
                    modifiers: _,
                },
            } => {
                data.key_downs.remove(&vkey);
                None
            }

            _ => None,
        }
    }
}

pub struct InputSystemShared {
    data: Arc<RwLock<InputSystemData>>,
}

impl InputSystemShared {
    fn new(data: Arc<RwLock<InputSystemData>>) -> Self {
        InputSystemShared { data: data }
    }

    /// Checks if a key is held down.
    #[inline(always)]
    pub fn is_key_down(&self, button: event::KeyboardButton) -> bool {
        self.data.read().unwrap().key_downs.contains(&button)
    }

    /// Checks if a key has been pressed on this frame.
    #[inline(always)]
    pub fn is_key_press(&self, button: event::KeyboardButton) -> bool {
        self.data.read().unwrap().key_presses.contains(&button)
    }

    /// Checks if a mouse button is held down.
    #[inline(always)]
    pub fn is_mouse_down(&self, button: event::MouseButton) -> bool {
        self.data.read().unwrap().mouse_downs.contains(&button)
    }

    /// Checks if a mouse button has been pressed on this frame.
    #[inline(always)]
    pub fn is_mouse_press(&self, button: event::MouseButton) -> bool {
        self.data.read().unwrap().mouse_presses.contains(&button)
    }

    /// Returns the mouse position relative to the top-left hand corner of the window.
    #[inline(always)]
    pub fn mouse_position(&self) -> (i32, i32) {
        self.data.read().unwrap().mouse_position
    }
}


struct InputSystemData {
    key_downs: HashSet<event::KeyboardButton>,
    key_presses: HashSet<event::KeyboardButton>,
    mouse_downs: HashSet<event::MouseButton>,
    mouse_presses: HashSet<event::MouseButton>,
    mouse_position: (i32, i32),
}

impl InputSystemData {
    fn new() -> Self {
        InputSystemData {
            key_downs: HashSet::new(),
            key_presses: HashSet::new(),
            mouse_downs: HashSet::new(),
            mouse_presses: HashSet::new(),
            mouse_position: (0, 0),
        }
    }

    fn reset(&mut self) {
        self.key_downs.clear();
        self.key_presses.clear();
        self.mouse_downs.clear();
        self.mouse_presses.clear();
    }
}