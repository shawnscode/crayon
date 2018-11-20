use math::prelude::Vector2;

use super::keyboard::Key;
use super::mouse::MouseButton;
use super::touchpad::TouchState;

/// Input device event, supports mouse and keyboard only.
#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    /// The cursor has moved on the window.
    /// The parameter are the (x, y) coords in pixels relative to the bottom-left
    /// corner of th window.
    MouseMoved { position: (f32, f32) },
    /// Pressed event on mouse has been received.
    MousePressed { button: MouseButton },
    /// Released event from mouse has been received.
    MouseReleased { button: MouseButton },
    /// A mouse wheel movement or touchpad scroll occurred.
    MouseWheel { delta: (f32, f32) },

    /// Pressed event on keyboard has been received.
    KeyboardPressed { key: Key },
    /// Released event from keyboard has been received.
    KeyboardReleased { key: Key },
    /// Received a unicode character.
    ReceivedCharacter { character: char },

    /// Represent touch event.
    ///
    /// Every time user touches screen new Start event with some finger id is generated. When
    /// the finger is removed from the screen End event with same id is generated.
    ///
    /// For every id there will be at least 2 events with phases Start and End (or Cancel).
    /// There may be 0 or more Move events.
    ///
    /// Depending on platform implementation id may or may not be reused by system after End event.
    Touch {
        id: u8,
        state: TouchState,
        position: Vector2<f32>,
    },
}
