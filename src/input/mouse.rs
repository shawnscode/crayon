use std::collections::HashSet;
use event;

pub struct Mouse {
    downs: HashSet<event::MouseButton>,
    presses: HashSet<event::MouseButton>,
    releases: HashSet<event::MouseButton>,
    last_position: (i32, i32),
    position: (i32, i32),
    scrol: (i32, i32),
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            downs: HashSet::new(),
            presses: HashSet::new(),
            releases: HashSet::new(),
            last_position: (0, 0),
            position: (0, 0),
            scrol: (0, 0),
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.downs.clear();
        self.presses.clear();
        self.releases.clear();
        self.last_position = (0, 0);
        self.position = (0, 0);
        self.scrol = (0, 0);
    }

    #[inline(always)]
    pub fn advance(&mut self) {
        self.presses.clear();
        self.releases.clear();
        self.scrol = (0, 0);
        self.last_position = self.position;
    }

    #[inline(always)]
    pub fn on_move(&mut self, position: (i32, i32)) {
        self.position = position;
    }

    #[inline(always)]
    pub fn on_button_pressed(&mut self, button: event::MouseButton) {
        if !self.downs.contains(&button) {
            self.downs.insert(button);
            self.presses.insert(button);
        }
    }

    #[inline(always)]
    pub fn on_button_released(&mut self, button: event::MouseButton) {
        self.downs.remove(&button);
        self.releases.insert(button);
    }

    #[inline(always)]
    pub fn on_wheel_scroll(&mut self, delta: (i32, i32)) {
        self.scrol = delta;
    }

    #[inline(always)]
    pub fn is_button_down(&self, button: event::MouseButton) -> bool {
        self.downs.contains(&button)
    }

    #[inline(always)]
    pub fn is_button_press(&self, button: event::MouseButton) -> bool {
        self.presses.contains(&button)
    }

    #[inline(always)]
    pub fn is_button_release(&self, button: event::MouseButton) -> bool {
        self.releases.contains(&button)
    }

    #[inline(always)]
    pub fn position(&self) -> (i32, i32) {
        self.position
    }

    #[inline(always)]
    pub fn movement(&self) -> (i32, i32) {
        (self.position.0 - self.last_position.0, self.position.1 - self.last_position.1)
    }

    #[inline(always)]
    pub fn scroll(&self) -> (i32, i32) {
        self.scrol
    }
}