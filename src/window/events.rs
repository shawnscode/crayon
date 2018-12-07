use crate::input::events::InputEvent;

/// The status of application.
#[derive(Debug, Clone, Copy)]
pub enum WindowEvent {
    /// The window has been woken up by another thread.
    Awakened,
    /// The window has been resumed.
    Resumed,
    /// The window has been suspended.
    Suspended,
    /// The window has been closed.
    Closed,
    /// The window gained focus of user input.
    GainFocus,
    /// The window lost focus of user input.
    LostFocus,
    /// The size of window has changed.
    Resized(u32, u32),
    /// The position of window has changed.
    Moved(u32, u32),
}

/// The enumerations of all events that come from various kinds of user input.
#[derive(Debug, Clone, Copy)]
pub enum Event {
    Window(WindowEvent),
    InputDevice(InputEvent),
}
