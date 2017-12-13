use std::collections::HashSet;
use event;

pub struct Keyboard {
    downs: HashSet<event::KeyboardButton>,
    presses: HashSet<event::KeyboardButton>,
    releases: HashSet<event::KeyboardButton>,
    chars: Vec<char>,
    max_chars: usize,
}

impl Keyboard {
    pub fn new(max_chars: usize) -> Self {
        Keyboard {
            downs: HashSet::new(),
            presses: HashSet::new(),
            releases: HashSet::new(),
            chars: Vec::with_capacity(max_chars),
            max_chars: max_chars,
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
    }

    #[inline(always)]
    pub fn on_key_pressed(&mut self, key: event::KeyboardButton) {
        if !self.downs.contains(&key) {
            self.downs.insert(key);
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
        if self.chars.len() < self.max_chars {
            self.chars.push(c);
        }
    }

    #[inline(always)]
    pub fn is_key_down(&self, key: event::KeyboardButton) -> bool {
        self.downs.contains(&key)
    }

    #[inline(always)]
    pub fn is_key_press(&self, key: event::KeyboardButton) -> bool {
        self.presses.contains(&key)
    }

    #[inline(always)]
    pub fn is_key_release(&self, key: event::KeyboardButton) -> bool {
        self.releases.contains(&key)
    }

    #[inline(always)]
    pub fn captured_chars(&self) -> &[char] {
        &self.chars
    }
}