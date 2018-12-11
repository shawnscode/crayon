use std::cmp::Ordering;
use std::time::Duration;

use crate::math::prelude::{MetricSpace, Vector2};
use crate::utils::time::Timestamp;

use super::MAX_TOUCHES;

/// The setup parameters of touch pad device.
///
/// Notes that the `distance` series paramters are measured in points.
#[derive(Debug, Clone, Copy)]
pub struct TouchPadParams {
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

impl Default for TouchPadParams {
    fn default() -> Self {
        TouchPadParams {
            min_pan_distance: 10.0,

            tap_timeout: Duration::from_millis(750),
            max_tap_distance: 30.0,

            touch_timeout: Duration::from_millis(250),
            max_touch_distance: 20.0,
        }
    }
}

/// Describes touch-screen input state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TouchState {
    Start,
    Move,
    End,
    Cancel,
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
    pub fn new(params: TouchPadParams) -> Self {
        TouchPad {
            record: TouchesRecord::default(),

            pan_detector: GesturePanDetector::new(params),
            pan: GesturePan::None,

            tap_detector: GestureTapDetector::new(1, params),
            tap: GestureTap::None,

            double_tap_detector: GestureTapDetector::new(2, params),
            double_tap: GestureTap::None,
        }
    }

    pub fn advance(&mut self) {
        self.pan = GesturePan::None;
        self.tap = GestureTap::None;
        self.double_tap = GestureTap::None;
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

    pub fn on_touch(&mut self, id: u8, state: TouchState, position: Vector2<f32>) {
        let touch = TouchEvent {
            id,
            state,
            position,
        };

        self.record.update_touch(touch);

        self.pan = self.pan_detector.detect(&self.record);
        self.tap = self.tap_detector.detect(&self.record);
        self.double_tap = self.double_tap_detector.detect(&self.record);
    }

    #[inline]
    pub fn is_touched(&self, index: usize) -> bool {
        self.record.position(index).is_some()
    }

    #[inline]
    pub fn position(&self, index: usize) -> Option<Vector2<f32>> {
        self.record.position(index)
    }

    #[inline]
    pub fn pan(&self) -> GesturePan {
        self.pan
    }

    #[inline]
    pub fn tap(&self) -> GestureTap {
        self.tap
    }

    #[inline]
    pub fn double_tap(&self) -> GestureTap {
        self.double_tap
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GestureTap {
    Action {
        /// The current tap position.
        position: Vector2<f32>,
    },
    None,
}

impl GestureTap {
    pub fn scale(&self, device_pixel_ratio: f32) -> GestureTap {
        match *self {
            GestureTap::Action { position } => GestureTap::Action {
                position: position * device_pixel_ratio,
            },

            GestureTap::None => GestureTap::None,
        }
    }
}

struct GestureTapDetector {
    record: TouchesRecord,

    last_tap_position: Vector2<f32>,
    last_tap_time: Timestamp,
    count: u32,

    required: u32,
    params: TouchPadParams,
}

impl GestureTapDetector {
    pub fn new(required: u32, params: TouchPadParams) -> Self {
        GestureTapDetector {
            record: TouchesRecord::default(),
            last_tap_position: Vector2::new(0.0, 0.0),
            last_tap_time: Timestamp::now(),
            count: 0,

            required,
            params,
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.record = TouchesRecord::default();
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
                // If multi-tap, checks if within max distance and tap timeout of
                // last tap, if not, start a new multitap sequence.
                if self.count > 0 {
                    if (ts - self.last_tap_time) > self.params.tap_timeout {
                        self.reset();
                    }

                    if t1.position.distance(self.last_tap_position) > self.params.max_tap_distance {
                        self.reset();
                    }
                }

                self.last_tap_position = t1.position;
                self.last_tap_time = ts;
                GestureTap::None
            }

            TouchState::End => {
                if (ts - self.last_tap_time) < self.params.touch_timeout
                    && t1.position.distance(self.last_tap_position) < self.params.max_touch_distance
                {
                    self.count += 1;
                    self.last_tap_position = t1.position;
                    self.last_tap_time = ts;

                    if self.count == self.required {
                        self.reset();
                        GestureTap::Action {
                            position: t1.position,
                        }
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
        start_position: Vector2<f32>,
    },
    Move {
        /// The start touch position.
        start_position: Vector2<f32>,
        /// The current touch position.
        position: Vector2<f32>,
        /// The movement during last frame.
        movement: Vector2<f32>,
    },
    End {
        /// The start touch position.
        start_position: Vector2<f32>,
        /// The current touch position.
        position: Vector2<f32>,
    },
    None,
}

impl GesturePan {
    pub fn scale(&self, device_pixel_ratio: f32) -> GesturePan {
        match *self {
            GesturePan::Start { start_position } => GesturePan::Start {
                start_position: start_position * device_pixel_ratio,
            },

            GesturePan::Move {
                start_position,
                position,
                movement,
            } => GesturePan::Move {
                start_position: start_position * device_pixel_ratio,
                position: position * device_pixel_ratio,
                movement: movement * device_pixel_ratio,
            },

            GesturePan::End {
                start_position,
                position,
            } => GesturePan::End {
                start_position: start_position * device_pixel_ratio,
                position: position * device_pixel_ratio,
            },

            GesturePan::None => GesturePan::None,
        }
    }
}

struct GesturePanDetector {
    position: Vector2<f32>,
    start_position: Vector2<f32>,
    pan: bool,
    record: TouchesRecord,

    params: TouchPadParams,
}

impl GesturePanDetector {
    pub fn new(params: TouchPadParams) -> Self {
        GesturePanDetector {
            params,
            position: Vector2::new(0.0, 0.0),
            start_position: Vector2::new(0.0, 0.0),
            pan: false,
            record: TouchesRecord::default(),
        }
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
                        movement,
                    }
                } else if self.start_position.distance(self.position)
                    >= self.params.min_pan_distance
                {
                    // Checks if min-distance is reached before starting panning.
                    self.pan = true;
                    GesturePan::Start {
                        start_position: self.start_position,
                    }
                } else {
                    GesturePan::None
                }
            }

            TouchState::End => if self.pan {
                self.position = t1.position;
                self.reset();
                GesturePan::End {
                    start_position: self.start_position,
                    position: self.position,
                }
            } else {
                self.reset();
                GesturePan::None
            },

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
struct TouchEvent {
    pub id: u8,
    pub state: TouchState,
    pub position: Vector2<f32>,
}

impl Default for TouchEvent {
    fn default() -> Self {
        TouchEvent {
            id: 0,
            state: TouchState::End,
            position: Vector2::new(0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TouchesRecord {
    touches: [(Timestamp, TouchEvent); MAX_TOUCHES + 1],
    len: usize,
}

impl Default for TouchesRecord {
    fn default() -> Self {
        let now = Timestamp::now();
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

    fn position(&self, index: usize) -> Option<Vector2<f32>> {
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
            self.touches[MAX_TOUCHES] = (Timestamp::now(), touch);
        }

        self.touches.sort_by(Self::sort);
        self.len = 0;

        for v in &self.touches {
            if v.1.state == TouchState::Start || v.1.state == TouchState::Move {
                self.len += 1;
            }
        }
    }

    fn sort(lhs: &(Timestamp, TouchEvent), rhs: &(Timestamp, TouchEvent)) -> Ordering {
        lhs.1.state.cmp(&rhs.1.state).then(lhs.0.cmp(&rhs.0))
    }
}
