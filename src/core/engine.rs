use std;
use std::any::{Any, TypeId};
use std::sync::RwLock;
use std::collections::{VecDeque, HashMap};
use std::time::{Duration, Instant};

///
pub trait Subsystem: Any + Send + Sync + 'static {}

/// `Engine` is the most foundamental struct used to manage all other subsystems.
pub struct Engine {
    min_fps: u32,
    max_fps: u32,
    max_inactive_fps: u32,
    smoothing_step: usize,
    previous_timesteps: VecDeque<Duration>,
    timestep: Duration,
    last_frame_timepoint: Instant,
    subsystems: HashMap<TypeId, Box<Any>>,
}

impl Engine {
    /// Constructs a new, empty engine.
    pub fn new() -> Self {
        Engine {
            min_fps: 0,
            max_fps: 0,
            max_inactive_fps: 0,
            smoothing_step: 0,
            previous_timesteps: VecDeque::new(),
            timestep: Duration::new(0, 0),
            last_frame_timepoint: Instant::now(),
            subsystems: HashMap::new(),
        }
    }

    /// Registers the system with the engine.
    #[inline]
    pub fn register<T>(&mut self, system: T)
        where T: Subsystem
    {
        self.subsystems.insert(TypeId::of::<T>(), Box::new(RwLock::new(system)));
    }

    // Retrieves reference to the specified subsystem. Panics if trying to get
    // non-exists subsystem.
    #[inline]
    pub fn get<T>(&self) -> &RwLock<T>
        where T: Subsystem
    {
        let boxed = self.subsystems
            .get(&TypeId::of::<T>())
            .expect("Tried to retrieve an non-exists subsystem.");
        boxed.downcast_ref().unwrap()
    }

    /// Performs one frame with specified fps. This will call update/render
    /// internally.
    pub fn run_one_frame(&mut self) -> Duration {
        // Perform waiting loop if maximum fps set, cooperatively gives up
        // a timeslice to the OS scheduler.
        if self.max_fps > 0 {
            let td = Duration::from_millis((1000 / self.max_fps) as u64);
            while self.last_frame_timepoint.elapsed() <= td {
                if (self.last_frame_timepoint.elapsed() + Duration::from_millis(5)) < td {
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
            elapsed = std::cmp::min(elapsed, Duration::from_millis((1000 / self.min_fps) as u64));
        }

        // Perform timestep smoothing.
        if self.smoothing_step > 0 {
            self.previous_timesteps.push_front(elapsed);
            if self.previous_timesteps.len() > self.smoothing_step {
                self.previous_timesteps.drain(self.smoothing_step..);

                self.timestep = Duration::new(0, 0);
                for step in self.previous_timesteps.iter() {
                    self.timestep += *step;
                }
                self.timestep /= self.previous_timesteps.len() as u32;
            } else {
                self.timestep = *self.previous_timesteps.front().unwrap();
            }
        } else {
            self.timestep = elapsed;
        }

        self.timestep
    }

    /// Set minimum frames per second. If fps goes lower than this, time will
    /// appear to slow. This is useful for some subsystems required strict minimum
    /// time step per frame, such like Collision checks.
    #[inline]
    pub fn set_min_fps(&mut self, fps: u32) {
        self.min_fps = fps;
    }

    /// Set maximum frames per second. The engine will sleep if fps is higher
    /// than this for less resource(e.g. power) consumptions.
    #[inline]
    pub fn set_max_fps(&mut self, fps: u32) {
        self.max_fps = fps;
    }

    /// Set maximum frames per second when the application does not have input
    /// focus.
    #[inline]
    pub fn set_max_inactive_fps(&mut self, fps: u32) {
        self.max_inactive_fps = fps;
    }

    /// Set how many frames to average for timestep smoothing.
    #[inline]
    pub fn set_time_smoothing_step(&mut self, step: u32) {
        self.smoothing_step = step as usize;
    }

    /// Get current fps.
    #[inline]
    pub fn get_fps(&self) -> u32 {
        if self.timestep.subsec_nanos() == 0 {
            0
        } else {
            (1000000000.0 / self.timestep.subsec_nanos() as f64) as u32
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {}

    struct CountSystem {
        value: u32,
    }

    impl Subsystem for CountSystem {}
    impl CountSystem {
        pub fn new() -> Self {
            CountSystem { value: 0 }
        }
        pub fn add(&mut self, v: u32) {
            self.value += v;
        }
    }

    #[test]
    fn subsystems() {
        let mut engine = Engine::new();
        engine.register(CountSystem::new());

        {
            let lock = engine.get::<CountSystem>();

            {
                let mut cs = lock.write().unwrap();
                assert_eq!(cs.value, 0);
                cs.add(32);
                assert_eq!(cs.value, 32);
            }

            {
                let cs = lock.read().unwrap();
                assert_eq!(cs.value, 32);
            }
        }

        engine.run_one_frame();
    }
}
