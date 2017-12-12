//! The enumerations of all events that come from various kinds of user input.

/// The status of application.
#[derive(Debug)]
pub enum ApplicationEvent {
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
}

/// Window related events.
#[derive(Debug)]
pub enum WindowEvent {
    /// The size of window has changed.
    Resized(u32, u32),
    /// The position of window has changed.
    Moved(u32, u32),
}

pub use glutin::VirtualKeyCode as KeyboardButton;
pub use glutin::MouseButton;

/// Input device event, supports mouse and keyboard only.
#[derive(Debug)]
pub enum InputDeviceEvent {
    /// The cursor has moved on the window.
    /// The parameter are the (x, y) coords in pixels relative to the top-left
    /// corner of th window.
    MouseMoved(i32, i32),
    /// Pressed event on mouse has been received.
    MousePressed(MouseButton),
    /// Released event from mouse has been received.
    MouseReleased(MouseButton),

    /// Pressed event on keyboard has been received.
    KeyboardPressed(KeyboardButton),
    /// Released event from keyboard has been received.
    KeyboardReleased(KeyboardButton),
}

#[derive(Debug)]
pub enum Event {
    Application(ApplicationEvent),
    Window(WindowEvent),
}