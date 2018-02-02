use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use math;
use math::MetricSpace;
use application::event;

/// The setup parameters of mouse device.
///
/// Notes that the `distance` series paramters will be multiplied by `HiDPI`
/// factor before recognizing processes.
#[derive(Debug, Clone, Copy)]
pub struct MouseSetup {
    pub press_timeout: Duration,
    pub max_press_distance: f32,

    pub click_timeout: Duration,
    pub max_click_distance: f32,
}

impl Default for MouseSetup {
    fn default() -> Self {
        MouseSetup {
            press_timeout: Duration::from_millis(500),
            max_press_distance: 25.0,

            click_timeout: Duration::from_millis(500),
            max_click_distance: 25.0,
        }
    }
}

pub struct Mouse {
    downs: HashSet<event::MouseButton>,
    presses: HashSet<event::MouseButton>,
    releases: HashSet<event::MouseButton>,
    last_position: math::Vector2<f32>,
    position: math::Vector2<f32>,
    scrol: math::Vector2<f32>,
    click_detectors: HashMap<event::MouseButton, ClickDetector>,
    setup: MouseSetup,
}

impl Mouse {
    pub fn new(setup: MouseSetup) -> Self {
        Mouse {
            downs: HashSet::new(),
            presses: HashSet::new(),
            releases: HashSet::new(),
            last_position: math::Vector2::new(0.0, 0.0),
            position: math::Vector2::new(0.0, 0.0),
            scrol: math::Vector2::new(0.0, 0.0),
            click_detectors: HashMap::new(),
            setup: setup,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.downs.clear();
        self.presses.clear();
        self.releases.clear();
        self.last_position = math::Vector2::new(0.0, 0.0);
        self.position = math::Vector2::new(0.0, 0.0);
        self.scrol = math::Vector2::new(0.0, 0.0);

        for v in self.click_detectors.values_mut() {
            v.reset();
        }
    }

    #[inline]
    pub fn advance(&mut self, hidpi: f32) {
        self.presses.clear();
        self.releases.clear();
        self.scrol = math::Vector2::new(0.0, 0.0);
        self.last_position = self.position;

        for v in self.click_detectors.values_mut() {
            v.advance(hidpi);
        }
    }

    #[inline]
    pub fn on_move(&mut self, position: (f32, f32)) {
        self.position = position.into();
    }

    #[inline]
    pub fn on_button_pressed(&mut self, button: event::MouseButton) {
        if !self.downs.contains(&button) {
            self.downs.insert(button);
            self.presses.insert(button);
        }

        if let Some(detector) = self.click_detectors.get_mut(&button) {
            detector.on_pressed(self.position);
            return;
        }

        let mut detector = ClickDetector::new(self.setup);
        detector.on_pressed(self.position);
        self.click_detectors.insert(button, detector);
    }

    #[inline]
    pub fn on_button_released(&mut self, button: event::MouseButton) {
        self.downs.remove(&button);
        self.releases.insert(button);

        if let Some(detector) = self.click_detectors.get_mut(&button) {
            detector.on_released(self.position);
            return;
        }

        let mut detector = ClickDetector::new(self.setup);
        detector.on_released(self.position);
        self.click_detectors.insert(button, detector);
    }

    #[inline]
    pub fn on_wheel_scroll(&mut self, delta: (f32, f32)) {
        self.scrol = delta.into();
    }

    #[inline]
    pub fn is_button_down(&self, button: event::MouseButton) -> bool {
        self.downs.contains(&button)
    }

    #[inline]
    pub fn is_button_press(&self, button: event::MouseButton) -> bool {
        self.presses.contains(&button)
    }

    #[inline]
    pub fn is_button_release(&self, button: event::MouseButton) -> bool {
        self.releases.contains(&button)
    }

    #[inline]
    pub fn is_button_click(&self, button: event::MouseButton) -> bool {
        if let Some(v) = self.click_detectors.get(&button) {
            v.clicks() > 0
        } else {
            false
        }
    }

    #[inline]
    pub fn is_button_double_click(&self, button: event::MouseButton) -> bool {
        if let Some(v) = self.click_detectors.get(&button) {
            v.clicks() > 0 && v.clicks() % 2 == 0
        } else {
            false
        }
    }

    #[inline]
    pub fn position(&self) -> math::Vector2<f32> {
        self.position
    }

    #[inline]
    pub fn movement(&self) -> math::Vector2<f32> {
        self.position - self.last_position
    }

    #[inline]
    pub fn scroll(&self) -> math::Vector2<f32> {
        self.scrol
    }
}

struct ClickDetector {
    last_press_time: Instant,
    last_press_position: math::Vector2<f32>,

    last_click_time: Instant,
    last_click_position: math::Vector2<f32>,

    clicks: u32,
    frame_clicks: u32,

    setup: MouseSetup,
    hidpi: f32,
}

impl ClickDetector {
    pub fn new(setup: MouseSetup) -> Self {
        ClickDetector {
            last_press_time: Instant::now(),
            last_press_position: math::Vector2::new(0.0, 0.0),

            last_click_time: Instant::now(),
            last_click_position: math::Vector2::new(0.0, 0.0),

            clicks: 0,
            frame_clicks: 0,

            setup: setup,
            hidpi: 1.0,
        }
    }

    pub fn reset(&mut self) {
        self.clicks = 0;
        self.frame_clicks = 0;
    }

    pub fn advance(&mut self, hidpi: f32) {
        self.frame_clicks = 0;
        self.hidpi = hidpi;
    }

    pub fn on_pressed(&mut self, position: math::Vector2<f32>) {
        // Store press down as start of a new potential click.
        let max_distance = self.setup.max_click_distance * self.hidpi;
        let timeout = self.setup.click_timeout;
        let now = Instant::now();

        // If multi-click, checks if within max distance and press timeout of
        // last click, if not, start a new multi-click sequence.
        if self.clicks > 0 {
            if (now - self.last_click_time) > timeout {
                self.reset();
            }

            if (position.distance(self.last_click_position)) > max_distance {
                self.reset();
            }
        }

        self.last_press_time = now;
        self.last_press_position = position;
    }

    pub fn on_released(&mut self, position: math::Vector2<f32>) {
        let max_distance = self.setup.max_press_distance * self.hidpi;
        let timeout = self.setup.press_timeout;
        let now = Instant::now();

        if (now - self.last_press_time) < timeout
            && (position.distance(self.last_press_position)) < max_distance
        {
            self.clicks += 1;
            self.frame_clicks = self.clicks;
            self.last_click_time = now;
            self.last_click_position = position;
        }
    }

    pub fn clicks(&self) -> u32 {
        self.frame_clicks
    }
}
