use std::collections::HashSet;
use math;
use event;

pub struct Mouse {
    downs: HashSet<event::MouseButton>,
    presses: HashSet<event::MouseButton>,
    releases: HashSet<event::MouseButton>,
    last_position: math::Vector2<f32>,
    position: math::Vector2<f32>,
    scrol: math::Vector2<f32>,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            downs: HashSet::new(),
            presses: HashSet::new(),
            releases: HashSet::new(),
            last_position: math::Vector2::new(0.0, 0.0),
            position: math::Vector2::new(0.0, 0.0),
            scrol: math::Vector2::new(0.0, 0.0),
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.downs.clear();
        self.presses.clear();
        self.releases.clear();
        self.last_position = math::Vector2::new(0.0, 0.0);
        self.position = math::Vector2::new(0.0, 0.0);
        self.scrol = math::Vector2::new(0.0, 0.0);
    }

    #[inline(always)]
    pub fn advance(&mut self) {
        self.presses.clear();
        self.releases.clear();
        self.scrol = math::Vector2::new(0.0, 0.0);
        self.last_position = self.position;
    }

    #[inline(always)]
    pub fn on_move(&mut self, position: (f32, f32)) {
        self.position = position.into();
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
    pub fn on_wheel_scroll(&mut self, delta: (f32, f32)) {
        self.scrol = delta.into();
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
    pub fn position(&self) -> math::Vector2<f32> {
        self.position
    }

    #[inline(always)]
    pub fn movement(&self) -> math::Vector2<f32> {
        self.position - self.last_position
    }

    #[inline(always)]
    pub fn scroll(&self) -> math::Vector2<f32> {
        self.scrol
    }
}