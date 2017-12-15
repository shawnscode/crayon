use std::time::{Instant, Duration};
use std::cmp::Ordering;

use application::event::{TouchState, TouchEvent};
use math;
use math::MetricSpace;

use super::MAX_TOUCHES;

/// The setup parameters of touch pad device.
///
/// Notes that the `distance` series paramters will be multiplied by HiDPI
/// factor before recognizing processes.
#[derive(Debug, Clone, Copy)]
pub struct TouchPadSetup {
    /// The minimum distance before a touch is recognized as panning.
    pub min_pan_distance: f32,
    /// The maximum time duration between two taps.
    pub tap_timeout: Duration,
    /// The maximum distance between two taps.
    pub max_tap_distance: f32,
    /// The maximum time duration between the touch pressing and releasing.
    pub touch_timeout: Duration,
    /// The minimum distance before a touch the touch pressing and releasing.
    pub max_touch_distance: f32,
}

impl Default for TouchPadSetup {
    fn default() -> Self {
        TouchPadSetup {
            min_pan_distance: 10.0,

            tap_timeout: Duration::from_millis(750),
            max_tap_distance: 30.0,

            touch_timeout: Duration::from_millis(250),
            max_touch_distance: 20.0,
        }
    }
}

pub struct TouchPad {
    record: TouchesRecord,

    pan_detector: GesturePanDetector,
    pan: GesturePan,

    tap_detector: GestureTapDetector,
    tap: GestureTap,

    double_tap_detector: GestureTapDetector,
    double_tap: GestureTap,
}

impl TouchPad {
    pub fn new(setup: TouchPadSetup) -> Self {
        TouchPad {
            record: TouchesRecord::default(),

            pan_detector: GesturePanDetector::new(setup),
            pan: GesturePan::None,

            tap_detector: GestureTapDetector::new(1, setup),
            tap: GestureTap::None,

            double_tap_detector: GestureTapDetector::new(2, setup),
            double_tap: GestureTap::None,
        }
    }

    pub fn advance(&mut self, hidpi: f32) {
        self.pan = GesturePan::None;
        self.pan_detector.set_hidpi_factor(hidpi);

        self.tap = GestureTap::None;
        self.tap_detector.set_hidpi_factor(hidpi);

        self.double_tap = GestureTap::None;
        self.double_tap_detector.set_hidpi_factor(hidpi);
    }

    pub fn reset(&mut self) {
        self.record = TouchesRecord::default();
        self.pan_detector.reset();
        self.pan = GesturePan::None;
        self.tap_detector.reset();
        self.tap = GestureTap::None;
        self.double_tap_detector.reset();
        self.double_tap = GestureTap::None;
    }

    pub fn on_touch(&mut self, touch: TouchEvent) {
        self.record.update_touch(touch);

        self.pan = self.pan_detector.detect(&self.record);
        self.tap = self.tap_detector.detect(&self.record);
        self.double_tap = self.double_tap_detector.detect(&self.record);
    }

    #[inline(always)]
    pub fn is_touched(&self, index: usize) -> bool {
        self.record.position(index).is_some()
    }

    #[inline(always)]
    pub fn position(&self, index: usize) -> Option<math::Vector2<f32>> {
        self.record.position(index)
    }

    #[inline(always)]
    pub fn pan(&self) -> GesturePan {
        self.pan
    }

    #[inline(always)]
    pub fn tap(&self) -> GestureTap {
        self.tap
    }

    #[inline(always)]
    pub fn double_tap(&self) -> GestureTap {
        self.double_tap
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GestureTap {
    Action {
        /// The current tap position.
        position: math::Vector2<f32>,
    },
    None,
}

struct GestureTapDetector {
    record: TouchesRecord,

    last_tap_position: math::Vector2<f32>,
    last_tap_time: Instant,
    count: u32,

    required: u32,
    hidpi: f32,
    setup: TouchPadSetup,
}

impl GestureTapDetector {
    pub fn new(required: u32, setup: TouchPadSetup) -> Self {
        GestureTapDetector {
            record: TouchesRecord::default(),
            last_tap_position: math::Vector2::new(0.0, 0.0),
            last_tap_time: Instant::now(),
            count: 0,

            required: required,
            hidpi: 1.0,
            setup: setup,
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.record = TouchesRecord::default();
    }

    pub fn set_hidpi_factor(&mut self, hidpi: f32) {
        self.hidpi = hidpi;
    }

    pub fn detect(&mut self, record: &TouchesRecord) -> GestureTap {
        let t1 = record.touches[0].1;
        let ts = record.touches[0].0;

        // Checks for required number of touches.
        if record.len != 1 {
            self.reset();
            return GestureTap::None;
        }

        // Checks if touch identifiers are unchanged (number of touches and same touch ids).
        if self.record.len > 0 && !self.record.is_same(record) {
            self.reset();
            return GestureTap::None;
        }

        self.record = *record;

        match t1.state {
            // Store touch down as start of a new potential tap.
            TouchState::Start => {
                let max_distance = self.setup.max_tap_distance * self.hidpi;
                let timeout = self.setup.tap_timeout;

                // If multi-tap, checks if within max distance and tap timeout of
                // last tap, if not, start a new multitap sequence.
                if self.count > 0 {
                    if (ts - self.last_tap_time) > timeout {
                        self.reset();
                    }

                    if t1.position.distance(self.last_tap_position) > max_distance {
                        self.reset();
                    }
                }

                self.last_tap_position = t1.position;
                self.last_tap_time = ts;
                GestureTap::None
            }

            TouchState::End => {
                let max_distance = self.setup.max_touch_distance * self.hidpi;
                let timeout = self.setup.touch_timeout;

                if (ts - self.last_tap_time) < timeout &&
                   t1.position.distance(self.last_tap_position) < max_distance {
                    self.count += 1;
                    self.last_tap_position = t1.position;
                    self.last_tap_time = ts;

                    if self.count == self.required {
                        self.reset();
                        GestureTap::Action { position: t1.position }
                    } else {
                        GestureTap::None
                    }
                } else {
                    self.reset();
                    GestureTap::None
                }
            }

            TouchState::Cancel => {
                self.reset();
                GestureTap::None
            }

            TouchState::Move => GestureTap::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GesturePan {
    Start {
        /// The start touch position.
        start_position: math::Vector2<f32>,
    },
    Move {
        /// The start touch position.
        start_position: math::Vector2<f32>,
        /// The current touch position.
        position: math::Vector2<f32>,
        /// The movement during last frame.
        movement: math::Vector2<f32>,
    },
    End {
        /// The start touch position.
        start_position: math::Vector2<f32>,
        /// The current touch position.
        position: math::Vector2<f32>,
    },
    None,
}

struct GesturePanDetector {
    position: math::Vector2<f32>,
    start_position: math::Vector2<f32>,
    pan: bool,
    record: TouchesRecord,

    hidpi: f32,
    setup: TouchPadSetup,
}

impl GesturePanDetector {
    pub fn new(setup: TouchPadSetup) -> Self {
        GesturePanDetector {
            position: math::Vector2::new(0.0, 0.0),
            start_position: math::Vector2::new(0.0, 0.0),
            pan: false,
            record: TouchesRecord::default(),

            hidpi: 1.0,
            setup: setup,
        }
    }

    pub fn set_hidpi_factor(&mut self, hidpi: f32) {
        self.hidpi = hidpi;
    }

    pub fn detect(&mut self, record: &TouchesRecord) -> GesturePan {
        let t1 = record.touches[0].1;

        // Checks for required number of touches.
        if record.len != 1 {
            self.reset();
            return GesturePan::None;
        }

        // Checks if touch identifiers are unchanged (number of touches and same touch ids).
        if self.record.len > 0 && !self.record.is_same(record) {
            self.reset();
            return GesturePan::None;
        }

        let min_distance = self.setup.min_pan_distance * self.hidpi;
        match t1.state {
            TouchState::Start => {
                self.record = *record;
                self.start_position = t1.position;
                self.position = self.start_position;
                self.pan = false;
                GesturePan::None
            }

            TouchState::Move => {
                let movement = t1.position - self.position;
                self.position = t1.position;

                if self.pan {
                    GesturePan::Move {
                        start_position: self.start_position,
                        position: self.position,
                        movement: movement,
                    }
                } else {
                    // Checks if min-distance is reached before starting panning.
                    if self.start_position.distance(self.position) < min_distance {
                        self.pan = true;
                        GesturePan::Start { start_position: self.start_position }
                    } else {
                        GesturePan::None
                    }
                }
            }

            TouchState::End => {
                if self.pan {
                    self.position = t1.position;
                    self.reset();
                    GesturePan::End {
                        start_position: self.start_position,
                        position: self.position,
                    }
                } else {
                    self.reset();
                    GesturePan::None
                }
            }

            TouchState::Cancel => {
                self.reset();
                GesturePan::None
            }
        }
    }

    pub fn reset(&mut self) {
        self.record.reset();
        self.pan = false;
    }
}

#[derive(Debug, Clone, Copy)]
struct TouchesRecord {
    touches: [(Instant, TouchEvent); MAX_TOUCHES + 1],
    len: usize,
}

impl Default for TouchesRecord {
    fn default() -> Self {
        let now = Instant::now();
        TouchesRecord {
            touches: [(now, TouchEvent::default()); MAX_TOUCHES + 1],
            len: 0,
        }
    }
}

impl TouchesRecord {
    fn reset(&mut self) {
        *self = TouchesRecord::default();
    }

    fn is_same(&self, rhs: &Self) -> bool {
        if self.len != rhs.len {
            return false;
        }

        for i in 0..self.len {
            if self.touches[i].1.id != rhs.touches[i].1.id {
                return false;
            }
        }

        true
    }

    fn position(&self, index: usize) -> Option<math::Vector2<f32>> {
        if self.len > index {
            Some(self.touches[index].1.position)
        } else {
            None
        }
    }

    fn update_touch(&mut self, touch: TouchEvent) {
        let mut found = false;
        for i in 0..self.len {
            if self.touches[i].1.id == touch.id {
                self.touches[i].1 = touch;
                found = true;
                break;
            }
        }

        if !found {
            self.touches[MAX_TOUCHES + 1] = (Instant::now(), touch);
        }

        self.touches.sort_by(Self::sort);
        self.len = 0;

        for v in &self.touches {
            if v.1.state == TouchState::Start || v.1.state == TouchState::Move {
                self.len += 1;
            }
        }
    }

    fn sort(lhs: &(Instant, TouchEvent), rhs: &(Instant, TouchEvent)) -> Ordering {
        lhs.1.state.cmp(&rhs.1.state).then(lhs.0.cmp(&rhs.0))
    }
}
