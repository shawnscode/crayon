use std::collections::{HashSet, HashMap};
use std::time::{Instant, Duration};

use application::event;

/// The setup parameters of keyboard device.
#[derive(Debug, Clone, Copy)]
pub struct KeyboardSetup {
    /// The maximum characters that could be captured in one frame.
    pub max_chars: usize,
    /// The time duration before a pressing is recognized as repeat operation.
    pub repeat_timeout: Duration,
    /// The interval time duration between triggering repeat events.
    pub repeat_interval_timeout: Duration,
}

impl Default for KeyboardSetup {
    fn default() -> Self {
        KeyboardSetup {
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
    downs: HashMap<event::KeyboardButton, KeyDownState>,
    presses: HashSet<event::KeyboardButton>,
    releases: HashSet<event::KeyboardButton>,
    chars: Vec<char>,
    setup: KeyboardSetup,
    now: Instant,
}

impl Keyboard {
    pub fn new(setup: KeyboardSetup) -> Self {
        Keyboard {
            downs: HashMap::new(),
            presses: HashSet::new(),
            releases: HashSet::new(),
            chars: Vec::with_capacity(setup.max_chars),
            setup: setup,
            now: Instant::now(),
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.downs.clear();
        self.presses.clear();
        self.releases.clear();
        self.chars.clear();
    }

    #[inline(always)]
    pub fn advance(&mut self) {
        self.presses.clear();
        self.releases.clear();
        self.chars.clear();

        let last_frame_ts = self.now;
        for (_, v) in &mut self.downs {
            match v {
                &mut KeyDownState::Start(ts) => {
                    if (last_frame_ts - ts) > self.setup.repeat_timeout {
                        *v = KeyDownState::Press(ts);
                    }
                }
                &mut KeyDownState::Press(ts) => {
                    if (last_frame_ts - ts) > self.setup.repeat_interval_timeout {
                        *v = KeyDownState::Press(last_frame_ts);
                    }
                }
            }
        }

        self.now = Instant::now();
    }

    #[inline(always)]
    pub fn on_key_pressed(&mut self, key: event::KeyboardButton) {
        if !self.downs.contains_key(&key) {
            self.downs.insert(key, KeyDownState::Start(self.now));
            self.presses.insert(key);
        }
    }

    #[inline(always)]
    pub fn on_key_released(&mut self, key: event::KeyboardButton) {
        self.downs.remove(&key);
        self.releases.insert(key);
    }

    #[inline(always)]
    pub fn on_char(&mut self, c: char) {
        if self.chars.len() < self.setup.max_chars {
            self.chars.push(c);
        }
    }

    #[inline(always)]
    pub fn is_key_down(&self, key: event::KeyboardButton) -> bool {
        self.downs.contains_key(&key)
    }

    #[inline(always)]
    pub fn is_key_press(&self, key: event::KeyboardButton) -> bool {
        self.presses.contains(&key)
    }

    #[inline(always)]
    pub fn is_key_release(&self, key: event::KeyboardButton) -> bool {
        self.releases.contains(&key)
    }

    pub fn is_key_repeat(&self, key: event::KeyboardButton) -> bool {
        if let Some(v) = self.downs.get(&key) {
            match *v {
                KeyDownState::Start(ts) => (self.now - ts) > self.setup.repeat_timeout,
                KeyDownState::Press(ts) => (self.now - ts) > self.setup.repeat_interval_timeout,
            }
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn captured_chars(&self) -> &[char] {
        &self.chars
    }
}