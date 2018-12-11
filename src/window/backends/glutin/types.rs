use glutin;

use super::super::super::events::{Event, WindowEvent};

use crate::input::events::InputEvent;
use crate::input::keyboard::Key;
use crate::input::mouse::MouseButton;
use crate::input::touchpad::TouchState;

use crate::math::prelude::Vector2;

pub fn from_event(source: glutin::Event, dimensions: Vector2<u32>) -> Option<Event> {
    match source {
        glutin::Event::WindowEvent { event, .. } => from_window_event(&event, dimensions),

        glutin::Event::Awakened => Some(Event::Window(WindowEvent::Awakened)),

        glutin::Event::Suspended(v) => if v {
            Some(Event::Window(WindowEvent::Suspended))
        } else {
            Some(Event::Window(WindowEvent::Resumed))
        },

        glutin::Event::DeviceEvent { .. } => None,
    }
}

fn from_window_event(source: &glutin::WindowEvent, dimensions: Vector2<u32>) -> Option<Event> {
    match *source {
        glutin::WindowEvent::CloseRequested => Some(Event::Window(WindowEvent::Closed)),

        glutin::WindowEvent::Focused(v) => if v {
            Some(Event::Window(WindowEvent::GainFocus))
        } else {
            Some(Event::Window(WindowEvent::LostFocus))
        },

        glutin::WindowEvent::Resized(glutin::dpi::LogicalSize { width, height }) => Some(
            Event::Window(WindowEvent::Resized(width as u32, height as u32)),
        ),

        glutin::WindowEvent::CursorMoved { position, .. } => {
            Some(Event::InputDevice(InputEvent::MouseMoved {
                position: (position.x as f32, dimensions.y as f32 - position.y as f32),
            }))
        }

        glutin::WindowEvent::MouseWheel { delta, .. } => match delta {
            glutin::MouseScrollDelta::LineDelta(x, y) => {
                Some(Event::InputDevice(InputEvent::MouseWheel {
                    delta: (x as f32, y as f32),
                }))
            }
            glutin::MouseScrollDelta::PixelDelta(pos) => {
                Some(Event::InputDevice(InputEvent::MouseWheel {
                    delta: (pos.x as f32, pos.y as f32),
                }))
            }
        },

        glutin::WindowEvent::MouseInput {
            state: glutin::ElementState::Pressed,
            button,
            ..
        } => Some(Event::InputDevice(InputEvent::MousePressed {
            button: button.into(),
        })),

        glutin::WindowEvent::MouseInput {
            state: glutin::ElementState::Released,
            button,
            ..
        } => Some(Event::InputDevice(InputEvent::MouseReleased {
            button: button.into(),
        })),

        glutin::WindowEvent::KeyboardInput {
            input:
                glutin::KeyboardInput {
                    state: glutin::ElementState::Pressed,
                    virtual_keycode: Some(key),
                    ..
                },
            ..
        } => from_virtual_key_code(key)
            .and_then(|key| Some(Event::InputDevice(InputEvent::KeyboardPressed { key }))),

        glutin::WindowEvent::KeyboardInput {
            input:
                glutin::KeyboardInput {
                    state: glutin::ElementState::Released,
                    virtual_keycode: Some(key),
                    ..
                },
            ..
        } => from_virtual_key_code(key)
            .and_then(|key| Some(Event::InputDevice(InputEvent::KeyboardReleased { key }))),

        glutin::WindowEvent::ReceivedCharacter(character) => {
            Some(Event::InputDevice(InputEvent::ReceivedCharacter {
                character,
            }))
        }

        glutin::WindowEvent::Touch(touch) => Some(Event::InputDevice(InputEvent::Touch {
            id: touch.id as u8,
            state: from_touch_state(touch.phase),
            position: (touch.location.x as f32, touch.location.y as f32).into(),
        })),

        _ => None,
    }
}

fn from_touch_state(state: glutin::TouchPhase) -> TouchState {
    match state {
        glutin::TouchPhase::Started => TouchState::Start,
        glutin::TouchPhase::Moved => TouchState::Move,
        glutin::TouchPhase::Ended => TouchState::End,
        glutin::TouchPhase::Cancelled => TouchState::Cancel,
    }
}

impl From<glutin::MouseButton> for MouseButton {
    fn from(mouse: glutin::MouseButton) -> Self {
        match mouse {
            glutin::MouseButton::Left => MouseButton::Left,
            glutin::MouseButton::Right => MouseButton::Right,
            glutin::MouseButton::Middle => MouseButton::Middle,
            glutin::MouseButton::Other(id) => MouseButton::Other(id),
        }
    }
}

fn from_virtual_key_code(key: glutin::VirtualKeyCode) -> Option<Key> {
    match key {
        glutin::VirtualKeyCode::Key1 => Some(Key::Key1),
        glutin::VirtualKeyCode::Key2 => Some(Key::Key2),
        glutin::VirtualKeyCode::Key3 => Some(Key::Key3),
        glutin::VirtualKeyCode::Key4 => Some(Key::Key4),
        glutin::VirtualKeyCode::Key5 => Some(Key::Key5),
        glutin::VirtualKeyCode::Key6 => Some(Key::Key6),
        glutin::VirtualKeyCode::Key7 => Some(Key::Key7),
        glutin::VirtualKeyCode::Key8 => Some(Key::Key8),
        glutin::VirtualKeyCode::Key9 => Some(Key::Key9),
        glutin::VirtualKeyCode::Key0 => Some(Key::Key0),
        glutin::VirtualKeyCode::A => Some(Key::A),
        glutin::VirtualKeyCode::B => Some(Key::B),
        glutin::VirtualKeyCode::C => Some(Key::C),
        glutin::VirtualKeyCode::D => Some(Key::D),
        glutin::VirtualKeyCode::E => Some(Key::E),
        glutin::VirtualKeyCode::F => Some(Key::F),
        glutin::VirtualKeyCode::G => Some(Key::G),
        glutin::VirtualKeyCode::H => Some(Key::H),
        glutin::VirtualKeyCode::I => Some(Key::I),
        glutin::VirtualKeyCode::J => Some(Key::J),
        glutin::VirtualKeyCode::K => Some(Key::K),
        glutin::VirtualKeyCode::L => Some(Key::L),
        glutin::VirtualKeyCode::M => Some(Key::M),
        glutin::VirtualKeyCode::N => Some(Key::N),
        glutin::VirtualKeyCode::O => Some(Key::O),
        glutin::VirtualKeyCode::P => Some(Key::P),
        glutin::VirtualKeyCode::Q => Some(Key::Q),
        glutin::VirtualKeyCode::R => Some(Key::R),
        glutin::VirtualKeyCode::S => Some(Key::S),
        glutin::VirtualKeyCode::T => Some(Key::T),
        glutin::VirtualKeyCode::U => Some(Key::U),
        glutin::VirtualKeyCode::V => Some(Key::V),
        glutin::VirtualKeyCode::W => Some(Key::W),
        glutin::VirtualKeyCode::X => Some(Key::X),
        glutin::VirtualKeyCode::Y => Some(Key::Y),
        glutin::VirtualKeyCode::Z => Some(Key::Z),
        glutin::VirtualKeyCode::Escape => Some(Key::Escape),
        glutin::VirtualKeyCode::F1 => Some(Key::F1),
        glutin::VirtualKeyCode::F2 => Some(Key::F2),
        glutin::VirtualKeyCode::F3 => Some(Key::F3),
        glutin::VirtualKeyCode::F4 => Some(Key::F4),
        glutin::VirtualKeyCode::F5 => Some(Key::F5),
        glutin::VirtualKeyCode::F6 => Some(Key::F6),
        glutin::VirtualKeyCode::F7 => Some(Key::F7),
        glutin::VirtualKeyCode::F8 => Some(Key::F8),
        glutin::VirtualKeyCode::F9 => Some(Key::F9),
        glutin::VirtualKeyCode::F10 => Some(Key::F10),
        glutin::VirtualKeyCode::F11 => Some(Key::F11),
        glutin::VirtualKeyCode::F12 => Some(Key::F12),
        glutin::VirtualKeyCode::F13 => Some(Key::F13),
        glutin::VirtualKeyCode::F14 => Some(Key::F14),
        glutin::VirtualKeyCode::F15 => Some(Key::F15),
        glutin::VirtualKeyCode::Snapshot => Some(Key::Snapshot),
        glutin::VirtualKeyCode::Scroll => Some(Key::Scroll),
        glutin::VirtualKeyCode::Pause => Some(Key::Pause),
        glutin::VirtualKeyCode::Insert => Some(Key::Insert),
        glutin::VirtualKeyCode::Home => Some(Key::Home),
        glutin::VirtualKeyCode::Delete => Some(Key::Delete),
        glutin::VirtualKeyCode::End => Some(Key::End),
        glutin::VirtualKeyCode::PageDown => Some(Key::PageDown),
        glutin::VirtualKeyCode::PageUp => Some(Key::PageUp),
        glutin::VirtualKeyCode::Left => Some(Key::Left),
        glutin::VirtualKeyCode::Up => Some(Key::Up),
        glutin::VirtualKeyCode::Right => Some(Key::Right),
        glutin::VirtualKeyCode::Down => Some(Key::Down),
        glutin::VirtualKeyCode::Back => Some(Key::Back),
        glutin::VirtualKeyCode::Return => Some(Key::Return),
        glutin::VirtualKeyCode::Space => Some(Key::Space),
        glutin::VirtualKeyCode::Compose => Some(Key::Compose),
        glutin::VirtualKeyCode::Caret => Some(Key::Caret),
        glutin::VirtualKeyCode::Numlock => Some(Key::Numlock),
        glutin::VirtualKeyCode::Numpad0 => Some(Key::Numpad0),
        glutin::VirtualKeyCode::Numpad1 => Some(Key::Numpad1),
        glutin::VirtualKeyCode::Numpad2 => Some(Key::Numpad2),
        glutin::VirtualKeyCode::Numpad3 => Some(Key::Numpad3),
        glutin::VirtualKeyCode::Numpad4 => Some(Key::Numpad4),
        glutin::VirtualKeyCode::Numpad5 => Some(Key::Numpad5),
        glutin::VirtualKeyCode::Numpad6 => Some(Key::Numpad6),
        glutin::VirtualKeyCode::Numpad7 => Some(Key::Numpad7),
        glutin::VirtualKeyCode::Numpad8 => Some(Key::Numpad8),
        glutin::VirtualKeyCode::Numpad9 => Some(Key::Numpad9),
        glutin::VirtualKeyCode::Add => Some(Key::Add),
        glutin::VirtualKeyCode::Backslash => Some(Key::Backslash),
        glutin::VirtualKeyCode::Calculator => Some(Key::Calculator),
        glutin::VirtualKeyCode::Capital => Some(Key::Capital),
        glutin::VirtualKeyCode::Colon => Some(Key::Colon),
        glutin::VirtualKeyCode::Comma => Some(Key::Comma),
        glutin::VirtualKeyCode::Convert => Some(Key::Convert),
        glutin::VirtualKeyCode::Decimal => Some(Key::Decimal),
        glutin::VirtualKeyCode::Divide => Some(Key::Divide),
        glutin::VirtualKeyCode::Equals => Some(Key::Equals),
        glutin::VirtualKeyCode::LAlt => Some(Key::LAlt),
        glutin::VirtualKeyCode::LBracket => Some(Key::LBracket),
        glutin::VirtualKeyCode::LControl => Some(Key::LControl),
        glutin::VirtualKeyCode::LShift => Some(Key::LShift),
        glutin::VirtualKeyCode::LWin => Some(Key::LWin),
        glutin::VirtualKeyCode::Minus => Some(Key::Minus),
        glutin::VirtualKeyCode::Multiply => Some(Key::Multiply),
        glutin::VirtualKeyCode::Mute => Some(Key::Mute),
        glutin::VirtualKeyCode::NavigateForward => Some(Key::NavigateForward),
        glutin::VirtualKeyCode::NavigateBackward => Some(Key::NavigateBackward),
        glutin::VirtualKeyCode::NumpadComma => Some(Key::NumpadComma),
        glutin::VirtualKeyCode::NumpadEnter => Some(Key::NumpadEnter),
        glutin::VirtualKeyCode::NumpadEquals => Some(Key::NumpadEquals),
        glutin::VirtualKeyCode::Period => Some(Key::Period),
        glutin::VirtualKeyCode::PlayPause => Some(Key::PlayPause),
        glutin::VirtualKeyCode::Power => Some(Key::Power),
        glutin::VirtualKeyCode::PrevTrack => Some(Key::PrevTrack),
        glutin::VirtualKeyCode::RAlt => Some(Key::RAlt),
        glutin::VirtualKeyCode::RBracket => Some(Key::RBracket),
        glutin::VirtualKeyCode::RControl => Some(Key::RControl),
        glutin::VirtualKeyCode::RShift => Some(Key::RShift),
        glutin::VirtualKeyCode::RWin => Some(Key::RWin),
        glutin::VirtualKeyCode::Semicolon => Some(Key::Semicolon),
        glutin::VirtualKeyCode::Slash => Some(Key::Slash),
        glutin::VirtualKeyCode::Sleep => Some(Key::Sleep),
        glutin::VirtualKeyCode::Stop => Some(Key::Stop),
        glutin::VirtualKeyCode::Subtract => Some(Key::Subtract),
        glutin::VirtualKeyCode::Tab => Some(Key::Tab),
        glutin::VirtualKeyCode::Underline => Some(Key::Underline),
        glutin::VirtualKeyCode::Unlabeled => Some(Key::Unlabeled),
        glutin::VirtualKeyCode::VolumeDown => Some(Key::VolumeDown),
        glutin::VirtualKeyCode::VolumeUp => Some(Key::VolumeUp),
        glutin::VirtualKeyCode::Wake => Some(Key::Wake),
        _ => None,
    }
}
