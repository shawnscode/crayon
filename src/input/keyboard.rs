use std::time::Duration;

use crate::utils::hash::{FastHashMap, FastHashSet};
use crate::utils::time::Timestamp;

/// The setup parameters of keyboard device.
#[derive(Debug, Clone, Copy)]
pub struct KeyboardParams {
    /// The maximum characters that could be captured in one frame.
    pub max_chars: usize,
    /// The time duration before a pressing is recognized as repeat operation.
    pub repeat_timeout: Duration,
    /// The interval time duration between triggering repeat events.
    pub repeat_interval_timeout: Duration,
}

impl Default for KeyboardParams {
    fn default() -> Self {
        KeyboardParams {
            max_chars: 128,
            repeat_timeout: Duration::from_millis(500),
            repeat_interval_timeout: Duration::from_millis(250),
        }
    }
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

enum KeyDownState {
    Start(Timestamp),
    Press(Timestamp),
}

pub struct Keyboard {
    downs: FastHashMap<Key, KeyDownState>,
    presses: FastHashSet<Key>,
    releases: FastHashSet<Key>,
    chars: Vec<char>,
    setup: KeyboardParams,
    now: Timestamp,
}

impl Keyboard {
    pub fn new(setup: KeyboardParams) -> Self {
        Keyboard {
            setup,
            downs: FastHashMap::default(),
            presses: FastHashSet::default(),
            releases: FastHashSet::default(),
            chars: Vec::with_capacity(setup.max_chars),
            now: Timestamp::now(),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.downs.clear();
        self.presses.clear();
        self.releases.clear();
        self.chars.clear();
    }

    #[inline]
    pub fn advance(&mut self) {
        self.presses.clear();
        self.releases.clear();
        self.chars.clear();

        let last_frame_ts = self.now;
        for v in self.downs.values_mut() {
            match *v {
                KeyDownState::Start(ts) => {
                    if (last_frame_ts - ts) > self.setup.repeat_timeout {
                        *v = KeyDownState::Press(ts);
                    }
                }
                KeyDownState::Press(ts) => {
                    if (last_frame_ts - ts) > self.setup.repeat_interval_timeout {
                        *v = KeyDownState::Press(last_frame_ts);
                    }
                }
            }
        }

        self.now = Timestamp::now();
    }

    #[inline]
    pub fn on_key_pressed(&mut self, key: Key) {
        let presses = &mut self.presses;
        let now = self.now;
        self.downs.entry(key).or_insert_with(|| {
            presses.insert(key);
            KeyDownState::Start(now)
        });
    }

    #[inline]
    pub fn on_key_released(&mut self, key: Key) {
        self.downs.remove(&key);
        self.releases.insert(key);
    }

    #[inline]
    pub fn on_char(&mut self, c: char) {
        if self.chars.len() < self.setup.max_chars {
            self.chars.push(c);
        }
    }

    #[inline]
    pub fn is_key_down(&self, key: Key) -> bool {
        self.downs.contains_key(&key)
    }

    #[inline]
    pub fn is_key_press(&self, key: Key) -> bool {
        self.presses.contains(&key)
    }
    #[inline]
    pub fn key_presses(&self) -> FastHashSet<Key> {
        self.presses.clone()
    }

    #[inline]
    pub fn is_key_release(&self, key: Key) -> bool {
        self.releases.contains(&key)
    }

    #[inline]
    pub fn key_releases(&self) -> FastHashSet<Key> {
        self.releases.clone()
    }
    
    pub fn is_key_repeat(&self, key: Key) -> bool {
        if let Some(v) = self.downs.get(&key) {
            match *v {
                KeyDownState::Start(ts) => (self.now - ts) > self.setup.repeat_timeout,
                KeyDownState::Press(ts) => (self.now - ts) > self.setup.repeat_interval_timeout,
            }
        } else {
            false
        }
    }

    #[inline]
    pub fn captured_chars(&self) -> &[char] {
        &self.chars
    }
}
