use math::prelude::Vector2;

/// The enumerations of all events that come from various kinds of user input.
#[derive(Debug, Clone, Copy)]
pub enum Event {
    Application(ApplicationEvent),
    InputDevice(InputDeviceEvent),
}

/// Symbolic name for a keyboard key.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum Key {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    Add,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Minus,
    Multiply,
    Mute,
    NavigateForward,  // also called "Prior"
    NavigateBackward, // also called "Next"
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
}

/// Describes a button of a mouse controller.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// The status of application.
#[derive(Debug, Clone, Copy)]
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
    /// The size of window has changed.
    Resized(u32, u32),
    /// The position of window has changed.
    Moved(u32, u32),
}

/// Input device event, supports mouse and keyboard only.
#[derive(Debug, Clone, Copy)]
pub enum InputDeviceEvent {
    /// The cursor has moved on the window.
    /// The parameter are the (x, y) coords in pixels relative to the top-left
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

/// Describes touch-screen input state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TouchState {
    Start,
    Move,
    End,
    Cancel,
}
