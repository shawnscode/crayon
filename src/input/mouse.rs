use std::time::Duration;

use crate::math::prelude::{MetricSpace, Vector2};
use crate::utils::hash::{FastHashMap, FastHashSet};
use crate::utils::time::Timestamp;

/// The setup parameters of mouse device.
///
/// Notes that the `distance` series paramters are measured in points.
#[derive(Debug, Clone, Copy)]
pub struct MouseParams {
    pub press_timeout: Duration,
    pub max_press_distance: f32,

    pub click_timeout: Duration,
    pub max_click_distance: f32,
}

impl Default for MouseParams {
    fn default() -> Self {
        MouseParams {
            press_timeout: Duration::from_millis(500),
            max_press_distance: 25.0,

            click_timeout: Duration::from_millis(500),
            max_click_distance: 25.0,
        }
    }
}

/// Describes a button of a mouse controller.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

pub struct Mouse {
    downs: FastHashSet<MouseButton>,
    presses: FastHashSet<MouseButton>,
    releases: FastHashSet<MouseButton>,
    last_position: Vector2<f32>,
    position: Vector2<f32>,
    scrol: Vector2<f32>,
    click_detectors: FastHashMap<MouseButton, ClickDetector>,
    params: MouseParams,
}

impl Mouse {
    pub fn new(params: MouseParams) -> Self {
        Mouse {
            params,
            downs: FastHashSet::default(),
            presses: FastHashSet::default(),
            releases: FastHashSet::default(),
            last_position: Vector2::new(0.0, 0.0),
            position: Vector2::new(0.0, 0.0),
            scrol: Vector2::new(0.0, 0.0),
            click_detectors: FastHashMap::default(),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.downs.clear();
        self.presses.clear();
        self.releases.clear();
        self.last_position = Vector2::new(0.0, 0.0);
        self.position = Vector2::new(0.0, 0.0);
        self.scrol = Vector2::new(0.0, 0.0);

        for v in self.click_detectors.values_mut() {
            v.reset();
        }
    }

    #[inline]
    pub fn advance(&mut self) {
        self.presses.clear();
        self.releases.clear();
        self.scrol = Vector2::new(0.0, 0.0);
        self.last_position = self.position;

        for v in self.click_detectors.values_mut() {
            v.advance();
        }
    }

    #[inline]
    pub fn on_move(&mut self, position: (f32, f32)) {
        self.position = position.into();
    }

    #[inline]
    pub fn on_button_pressed(&mut self, button: MouseButton) {
        if !self.downs.contains(&button) {
            self.downs.insert(button);
            self.presses.insert(button);
        }

        if let Some(detector) = self.click_detectors.get_mut(&button) {
            detector.on_pressed(self.position);
            return;
        }

        let mut detector = ClickDetector::new(self.params);
        detector.on_pressed(self.position);
        self.click_detectors.insert(button, detector);
    }
    #[inline]
    pub fn mouse_presses(&self) -> FastHashSet<MouseButton> {
        self.presses.clone()
    }
    #[inline]
    pub fn on_button_released(&mut self, button: MouseButton) {
        self.downs.remove(&button);
        self.releases.insert(button);

        if let Some(detector) = self.click_detectors.get_mut(&button) {
            detector.on_released(self.position);
            return;
        }

        let mut detector = ClickDetector::new(self.params);
        detector.on_released(self.position);
        self.click_detectors.insert(button, detector);
    }
    #[inline]
    pub fn mouse_releases(&self) -> FastHashSet<MouseButton> {
        self.releases.clone()
    }
    #[inline]
    pub fn on_wheel_scroll(&mut self, delta: (f32, f32)) {
        self.scrol = delta.into();
    }

    #[inline]
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.downs.contains(&button)
    }

    #[inline]
    pub fn is_button_press(&self, button: MouseButton) -> bool {
        self.presses.contains(&button)
    }

    #[inline]
    pub fn button_presses(&self) -> FastHashSet<MouseButton> {
        self.presses.clone()
    }

    #[inline]
    pub fn is_button_release(&self, button: MouseButton) -> bool {
        self.releases.contains(&button)
    }

    #[inline]
    pub fn button_releases(&self) -> FastHashSet<MouseButton> {
        self.releases.clone()
    }

    #[inline]
    pub fn is_button_click(&self, button: MouseButton) -> bool {
        if let Some(v) = self.click_detectors.get(&button) {
            v.clicks() > 0
        } else {
            false
        }
    }

    #[inline]
    pub fn is_button_double_click(&self, button: MouseButton) -> bool {
        if let Some(v) = self.click_detectors.get(&button) {
            v.clicks() > 0 && v.clicks() % 2 == 0
        } else {
            false
        }
    }

    #[inline]
    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    #[inline]
    pub fn movement(&self) -> Vector2<f32> {
        self.position - self.last_position
    }

    #[inline]
    pub fn scroll(&self) -> Vector2<f32> {
        self.scrol
    }
}

struct ClickDetector {
    last_press_time: Timestamp,
    last_press_position: Vector2<f32>,

    last_click_time: Timestamp,
    last_click_position: Vector2<f32>,

    clicks: u32,
    frame_clicks: u32,

    params: MouseParams,
}

impl ClickDetector {
    pub fn new(params: MouseParams) -> Self {
        ClickDetector {
            params,

            last_press_time: Timestamp::now(),
            last_press_position: Vector2::new(0.0, 0.0),

            last_click_time: Timestamp::now(),
            last_click_position: Vector2::new(0.0, 0.0),

            clicks: 0,
            frame_clicks: 0,
        }
    }

    pub fn reset(&mut self) {
        self.clicks = 0;
        self.frame_clicks = 0;
    }

    pub fn advance(&mut self) {
        self.frame_clicks = 0;
    }

    pub fn on_pressed(&mut self, position: Vector2<f32>) {
        // Store press down as start of a new potential click.
        let now = Timestamp::now();

        // If multi-click, checks if within max distance and press timeout of
        // last click, if not, start a new multi-click sequence.
        if self.clicks > 0 {
            if (now - self.last_click_time) > self.params.click_timeout {
                self.reset();
            }

            if (position.distance(self.last_click_position)) > self.params.max_click_distance {
                self.reset();
            }
        }

        self.last_press_time = now;
        self.last_press_position = position;
    }

    pub fn on_released(&mut self, position: Vector2<f32>) {
        let now = Timestamp::now();

        if (now - self.last_press_time) < self.params.press_timeout
            && (position.distance(self.last_press_position)) < self.params.max_press_distance
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
