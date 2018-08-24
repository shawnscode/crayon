//! Timing and stepping system.

use std;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use application::settings::EngineParams;

/// `TimeSystem`
pub struct TimeSystem {
    min_fps: u32,
    max_fps: u32,
    max_inactive_fps: u32,
    smoothing_step: usize,

    timestep: Duration,
    previous_timesteps: VecDeque<Duration>,
    last_frame_timepoint: Instant,
    shared: Arc<TimeSystemShared>,
}

impl TimeSystem {
    /// Creates a `TimeSystem` from settings.
    pub fn new(setup: EngineParams) -> Self {
        let shared = TimeSystemShared::new(setup);
        TimeSystem {
            min_fps: setup.min_fps,
            max_fps: setup.max_fps,
            max_inactive_fps: setup.max_inactive_fps,
            smoothing_step: setup.time_smooth_step as usize,
            previous_timesteps: VecDeque::new(),
            timestep: Duration::new(0, 0),
            last_frame_timepoint: Instant::now(),
            shared: Arc::new(shared),
        }
    }

    /// Gets the multi-thread friendly parts of `TimeSystem`.
    pub fn shared(&self) -> Arc<TimeSystemShared> {
        self.shared.clone()
    }

    pub(crate) fn advance(&mut self) -> Duration {
        // Synchonize with configurations.
        self.min_fps = *self.shared.min_fps.read().unwrap();
        self.max_fps = *self.shared.max_fps.read().unwrap();
        self.max_inactive_fps = *self.shared.max_inactive_fps.read().unwrap();
        self.smoothing_step = *self.shared.smoothing_step.read().unwrap();

        // Perform waiting loop if maximum fps set, cooperatively gives up
        // a timeslice to the OS scheduler.
        if self.max_fps > 0 {
            let td = Duration::from_millis(u64::from(1000 / self.max_fps));
            while self.last_frame_timepoint.elapsed() <= td {
                if (self.last_frame_timepoint.elapsed() + Duration::from_millis(2)) < td {
                    std::thread::sleep(Duration::from_millis(1));
                } else {
                    std::thread::yield_now();
                }
            }
        }

        let mut elapsed = self.last_frame_timepoint.elapsed();
        self.last_frame_timepoint = Instant::now();

        // If fps lower than minimum, simply clamp it.
        if self.min_fps > 0 {
            elapsed = std::cmp::min(
                elapsed,
                Duration::from_millis(u64::from(1000 / self.min_fps)),
            );
        }

        // Perform timestep smoothing.
        if self.smoothing_step > 0 {
            self.previous_timesteps.push_front(elapsed);
            if self.previous_timesteps.len() > self.smoothing_step {
                self.previous_timesteps.drain(self.smoothing_step..);

                self.timestep = Duration::new(0, 0);
                for step in &self.previous_timesteps {
                    self.timestep += *step;
                }
                self.timestep /= self.previous_timesteps.len() as u32;
            } else {
                self.timestep = *self.previous_timesteps.front().unwrap();
            }
        } else {
            self.timestep = elapsed;
        }

        *self.shared.timestep.write().unwrap() = self.timestep;
        self.timestep
    }
}

/// The multi-thread friendly parts of `TimeSystem`.
pub struct TimeSystemShared {
    min_fps: RwLock<u32>,
    max_fps: RwLock<u32>,
    max_inactive_fps: RwLock<u32>,
    smoothing_step: RwLock<usize>,
    timestep: RwLock<Duration>,
}

impl TimeSystemShared {
    pub fn new(setup: EngineParams) -> Self {
        TimeSystemShared {
            min_fps: RwLock::new(setup.min_fps),
            max_fps: RwLock::new(setup.max_fps),
            max_inactive_fps: RwLock::new(setup.max_inactive_fps),
            smoothing_step: RwLock::new(setup.time_smooth_step as usize),
            timestep: RwLock::new(Duration::new(0, 0)),
        }
    }

    /// Set minimum frames per second. If fps goes lower than this, time will
    /// appear to slow. This is useful for some subsystems required strict minimum
    /// time step per frame, such like Collision checks.
    #[inline]
    pub fn set_min_fps(&self, fps: u32) {
        *self.min_fps.write().unwrap() = fps;
    }

    /// Set maximum frames per second. The engine will sleep if fps is higher
    /// than this for less resource(e.g. power) consumptions.
    #[inline]
    pub fn set_max_fps(&self, fps: u32) {
        *self.max_fps.write().unwrap() = fps;
    }

    /// Set maximum frames per second when the application does not have input
    /// focus.
    #[inline]
    pub fn set_max_inactive_fps(&self, fps: u32) {
        *self.max_inactive_fps.write().unwrap() = fps;
    }

    /// Set how many frames to average for timestep smoothing.
    #[inline]
    pub fn set_time_smoothing_step(&mut self, step: u32) {
        *self.smoothing_step.write().unwrap() = step as usize;
    }

    /// Gets current fps.
    #[inline]
    pub fn get_fps(&self) -> u32 {
        let ts = self.timestep.read().unwrap();
        if ts.subsec_nanos() == 0 {
            0
        } else {
            (1000000000.0 / f64::from(ts.subsec_nanos())) as u32
        }
    }

    /// Gets the duration duraing last frame.
    #[inline]
    pub fn frame_delta(&self) -> Duration {
        *self.timestep.read().unwrap()
    }
}
