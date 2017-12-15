use std::sync::{Arc, RwLock};
use std::time::{Instant, Duration};
use std::sync::mpsc;
use rayon;

use super::*;
use graphics;
use resource;
use input;
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

impl ContextSystem for time::TimeSystem {
    type Shared = time::TimeSystemShared;
}

/// `Engine` is the root object of the game application. It binds various sub-systems in
/// a central place and takes take of trivial tasks like the execution order or life-time
/// management.
pub struct Engine {
    scheduler: rayon::ThreadPool,

    pub events_loop: event::EventsLoop,
    pub input: input::InputSystem,
    pub window: Arc<graphics::Window>,
    pub graphics: graphics::GraphicsSystem,
    pub resource: resource::ResourceSystem,
    pub time: time::TimeSystem,

    context: Arc<Context>,
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

        let input = input::InputSystem::new(settings.input);
        let input_shared = input.shared();

        let events_loop = event::EventsLoop::new();
        let window = Arc::new(wb.build(&events_loop.underlaying())?);

        let resource = resource::ResourceSystem::new()?;
        let resource_shared = resource.shared();

        let graphics = graphics::GraphicsSystem::new(window.clone(), resource_shared.clone())?;
        let graphics_shared = graphics.shared();

        let time = time::TimeSystem::new(settings.engine)?;
        let time_shared = time.shared();

        let confs = rayon::Configuration::new();
        let scheduler = rayon::ThreadPool::new(confs).unwrap();

        let mut context = Context::new();
        context.insert::<resource::ResourceSystem>(resource_shared);
        context.insert::<graphics::GraphicsSystem>(graphics_shared);
        context.insert::<input::InputSystem>(input_shared);
        context.insert::<time::TimeSystem>(time_shared);

        Ok(Engine {
               scheduler: scheduler,

               events_loop: events_loop,
               input: input,
               window: window,
               graphics: graphics,
               resource: resource,
               time: time,

               context: Arc::new(context),
           })
    }

    pub fn context(&self) -> &Context {
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

        let mut alive = true;
        'main: while alive {
            self.input.advance(self.window.hidpi_factor());

            // Poll any possible events first.
            for v in self.events_loop.advance() {
                match *v {
                    event::Event::Application(value) => {
                        {
                            let mut application = application.write().unwrap();
                            application.on_receive_event(&self.context, value)?;
                        }

                        match value {
                            event::ApplicationEvent::Closed => {
                                alive = false;
                            }
                            _ => {}
                        };
                    }

                    event::Event::InputDevice(value) => self.input.update_with(value),
                }
            }

            self.time.advance();
            self.graphics.swap_frames();

            let (video_info, duration) = {
                let application = application.clone();
                let (rx, tx) = mpsc::channel();

                let ctx = self.context.clone();
                let closure = move || {
                    let v = Engine::execute_frame(&ctx, application);
                    rx.send(v).unwrap();
                };

                // Perform update and render submitting for frame [x], and drawing
                // frame [x-1] at the same time.
                self.scheduler.spawn(closure);

                // This will block the main-thread until all the graphics commands
                // is finished by GPU.
                let video_info = self.graphics.advance()?;
                let duration = tx.recv().unwrap()?;
                (video_info, duration)
            };

            {
                let info = FrameInfo {
                    video: video_info,
                    duration: duration,
                    fps: self.time.shared().get_fps(),
                };

                let ctx = self.context.clone();
                let application = application.clone();

                let closure = || {
                    let mut application = application.write().unwrap();
                    application.on_post_update(&ctx, &info)
                };

                self.scheduler.install(closure)?;
            }

            alive = alive || !self.context.is_shutdown();
        }

        {
            let mut application = application.write().unwrap();
            application.on_exit(&self.context)?;
        }

        Ok(self)
    }

    fn execute_frame(ctx: &Context, application: Arc<RwLock<Application>>) -> Result<Duration> {
        let ts = Instant::now();

        let mut application = application.write().unwrap();
        application.on_update(&ctx)?;
        application.on_render(&ctx)?;

        Ok(Instant::now() - ts)
    }
}