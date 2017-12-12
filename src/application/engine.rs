use std;
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use rayon;

use super::*;
use graphics;
use resource;
use input;
use input::event;
use super::context::{Context, ContextSystem};

impl ContextSystem for resource::ResourceSystem {
    type Shared = resource::ResourceSystemShared;
}

impl ContextSystem for graphics::GraphicsSystem {
    type Shared = graphics::GraphicsSystemShared;
}

impl ContextSystem for input::InputSystem {
    type Shared = input::InputSystemShared;
}

/// `Engine` is the root object of the game application. It binds various sub-systems in
/// a central place and takes take of trivial tasks like the execution order or life-time
/// management.
pub struct Engine {
    min_fps: u32,
    max_fps: u32,
    max_inactive_fps: u32,
    smoothing_step: usize,
    previous_timesteps: VecDeque<Duration>,
    timestep: Duration,
    last_frame_timepoint: Instant,
    scheduler: rayon::ThreadPool,

    pub input: input::InputSystem,
    pub window: Arc<graphics::Window>,
    pub graphics: graphics::GraphicsSystem,
    pub resource: resource::ResourceSystem,

    context: Arc<RwLock<Context>>,
}

impl Engine {
    /// Constructs a new, empty engine.
    pub fn new() -> Result<Self> {
        Engine::new_with(Settings::default())
    }

    /// Setup engine with specified settings.
    pub fn new_with(settings: Settings) -> Result<Self> {
        let mut wb = graphics::WindowBuilder::new();
        wb.with_title(settings.window.title.clone())
            .with_dimensions(settings.window.width, settings.window.height);

        let input = input::InputSystem::new();
        let input_shared = input.shared();
        let window = Arc::new(wb.build(&input)?);

        let resource = resource::ResourceSystem::new()?;
        let resource_shared = resource.shared();

        let graphics = graphics::GraphicsSystem::new(window.clone(), resource_shared.clone())?;
        let graphics_shared = graphics.shared();

        let confs = rayon::Configuration::new();
        let scheduler = rayon::ThreadPool::new(confs).unwrap();

        let mut context = Context::new();
        context.insert::<resource::ResourceSystem>(resource_shared);
        context.insert::<graphics::GraphicsSystem>(graphics_shared);
        context.insert::<input::InputSystem>(input_shared);

        Ok(Engine {
               min_fps: settings.engine.min_fps,
               max_fps: settings.engine.max_fps,
               max_inactive_fps: settings.engine.max_inactive_fps,
               smoothing_step: settings.engine.time_smooth_step as usize,
               previous_timesteps: VecDeque::new(),
               timestep: Duration::new(0, 0),
               last_frame_timepoint: Instant::now(),
               scheduler: scheduler,

               input: input,
               window: window,
               graphics: graphics,
               resource: resource,
               context: Arc::new(RwLock::new(context)),
           })
    }

    pub fn context(&self) -> &Arc<RwLock<Context>> {
        &self.context
    }

    /// Run the main loop of `Engine`, this will block the working
    /// thread until we finished.
    pub fn run<T>(mut self, application: T) -> Result<Self>
        where T: Application + Send + Sync + 'static
    {
        let application = Arc::new(RwLock::new(application));

        let dir = ::std::env::current_dir()?;
        println!("Run crayon-runtim with working directory {:?}.", dir);

        let mut events = Vec::new();
        let mut alive = true;
        'main: while alive {
            // Poll any possible events first.
            events.clear();

            self.input.run_one_frame(&mut events);
            for v in events.drain(..) {
                match v {
                    event::Event::Application(value) => {
                        match value {
                            event::ApplicationEvent::Closed => {
                                alive = false;
                            }
                            other => println!("Drop {:?}.", other),
                        };
                    }
                    other => println!("Drop {:?}.", other),
                }
            }

            self.advance();
            self.graphics.swap_frames();

            {
                let mut ctx = self.context.write().unwrap();
                ctx.set_frame_delta(self.timestep);
            }

            let (video_info, duration) = {
                let application = application.clone();
                let (rx, tx) = mpsc::channel();

                let ctx = self.context.clone();
                let closure = move || {
                    let ctx = ctx.read().unwrap();
                    let v = Engine::execute_frame(&ctx, application);
                    rx.send(v).unwrap();
                };

                // Perform update and render submitting for frame [x], and drawing
                // frame [x-1] at the same time.
                self.scheduler.spawn(closure);

                // This will block the main-thread until all the graphics commands
                // is finished by GPU.
                let video_info = self.graphics.advance().unwrap();
                let duration = tx.recv().unwrap()?;
                (video_info, duration)
            };

            {
                let info = FrameInfo {
                    video: video_info,
                    duration: duration,
                    fps: self.get_fps(),
                };

                let ctx = self.context.clone();
                let application = application.clone();

                let closure = || {
                    let ctx = ctx.read().unwrap();
                    let mut application = application.write().unwrap();
                    application.on_post_update(&ctx, &info)
                };

                self.scheduler.install(closure)?;
            }

            alive = alive || !self.context.read().unwrap().is_shutdown();
        }

        {
            let mut application = application.write().unwrap();
            let ctx = self.context.read().unwrap();
            application.on_exit(&ctx).unwrap();
        }

        Ok(self)
    }

    fn execute_frame(ctx: &Context,
                     application: Arc<RwLock<Application>>)
                     -> Result<time::Duration> {
        let ts = time::Instant::now();
        let mut application = application.write().unwrap();
        application.on_update(&ctx)?;
        application.on_render(&ctx)?;

        Ok(time::Instant::now() - ts)
    }

    /// Advance one frame.
    pub fn advance(&mut self) -> Duration {
        // Perform waiting loop if maximum fps set, cooperatively gives up
        // a timeslice to the OS scheduler.
        if self.max_fps > 0 {
            let td = Duration::from_millis((1000 / self.max_fps) as u64);
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

    /// Returns timestep of last frame.
    #[inline]
    pub fn timestep_in_seconds(&self) -> f32 {
        let sec = self.timestep.as_secs();
        let nansec = self.timestep.subsec_nanos() as u64;
        sec as f32 + (nansec as f32 * 1e-9)
    }
}