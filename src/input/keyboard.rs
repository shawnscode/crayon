use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub use application::event::KeyboardButton;

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

enum KeyDownState {
    Start(Instant),
    Press(Instant),
}

pub struct Keyboard {
    downs: HashMap<KeyboardButton, KeyDownState>,
    presses: HashSet<KeyboardButton>,
    releases: HashSet<KeyboardButton>,
    chars: Vec<char>,
    setup: KeyboardParams,
    now: Instant,
}

impl Keyboard {
    pub fn new(setup: KeyboardParams) -> Self {
        Keyboard {
            downs: HashMap::new(),
            presses: HashSet::new(),
            releases: HashSet::new(),
            chars: Vec::with_capacity(setup.max_chars),
            setup: setup,
            now: Instant::now(),
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
                KeyDownState::Start(ts) => if (last_frame_ts - ts) > self.setup.repeat_timeout {
                    *v = KeyDownState::Press(ts);
                },
                KeyDownState::Press(ts) => {
                    if (last_frame_ts - ts) > self.setup.repeat_interval_timeout {
                        *v = KeyDownState::Press(last_frame_ts);
                    }
                }
            }
        }

        self.now = Instant::now();
    }

    #[inline]
    pub fn on_key_pressed(&mut self, key: KeyboardButton) {
        if !self.downs.contains_key(&key) {
            self.presses.insert(key);
            self.downs.insert(key, KeyDownState::Start(self.now));
        }
    }

    #[inline]
    pub fn on_key_released(&mut self, key: KeyboardButton) {
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
    pub fn is_key_down(&self, key: KeyboardButton) -> bool {
        self.downs.contains_key(&key)
    }

    #[inline]
    pub fn is_key_press(&self, key: KeyboardButton) -> bool {
        self.presses.contains(&key)
    }

    #[inline]
    pub fn is_key_release(&self, key: KeyboardButton) -> bool {
        self.releases.contains(&key)
    }

    pub fn is_key_repeat(&self, key: KeyboardButton) -> bool {
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
