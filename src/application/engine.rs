use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use super::context::{Context, ContextSystem};
use super::*;
use graphics;
use input;
use resource;

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
    pub events_loop: event::EventsLoop,
    pub window: Arc<graphics::window::Window>,

    pub input: input::InputSystem,
    pub graphics: graphics::GraphicsSystem,
    pub resource: resource::ResourceSystem,
    pub time: time::TimeSystem,

    context: Arc<Context>,
    headless: bool,
}

impl Engine {
    /// Constructs a new, empty engine.
    pub fn new() -> Result<Self> {
        Engine::new_with(&Settings::default())
    }

    /// Setup engine with specified settings.
    pub fn new_with(settings: &Settings) -> Result<Self> {
        let mut wb = graphics::window::WindowBuilder::new();
        wb.with_title(settings.window.title.clone())
            .with_dimensions(settings.window.width, settings.window.height);

        let input = input::InputSystem::new(settings.input);
        let input_shared = input.shared();

        let events_loop = event::EventsLoop::new();
        let window = Arc::new(wb.build(events_loop.underlaying())?);

        let resource = resource::ResourceSystem::new()?;
        let resource_shared = resource.shared();

        let graphics = graphics::GraphicsSystem::new(window.clone(), resource_shared.clone())?;
        let graphics_shared = graphics.shared();

        let time = time::TimeSystem::new(settings.engine)?;
        let time_shared = time.shared();

        let mut context = Context::new();
        context.insert::<resource::ResourceSystem>(resource_shared);
        context.insert::<graphics::GraphicsSystem>(graphics_shared);
        context.insert::<input::InputSystem>(input_shared);
        context.insert::<time::TimeSystem>(time_shared);

        Ok(Engine {
            events_loop: events_loop,
            input: input,
            window: window,
            graphics: graphics,
            resource: resource,
            time: time,

            context: Arc::new(context),
            headless: settings.headless,
        })
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Run the main loop of `Engine`, this will block the working
    /// thread until we finished.
    pub fn run<T>(mut self, application: T) -> Result<Self>
    where
        T: Application + Send + Sync + 'static,
    {
        let application = Arc::new(RwLock::new(application));

        let dir = ::std::env::current_dir()?;
        println!("Run crayon-runtim with working directory {:?}.", dir);

        let (task_sender, task_receiver) = mpsc::channel();
        let (join_sender, join_receiver) = mpsc::channel();
        Self::main_thread(
            task_receiver,
            join_sender,
            self.context.clone(),
            application.clone(),
        );

        let mut alive = true;
        while alive {
            self.input.advance(self.window.hidpi_factor());

            // Poll any possible events first.
            for v in self.events_loop.advance() {
                match *v {
                    event::Event::Application(value) => {
                        {
                            let mut application = application.write().unwrap();
                            application.on_receive_event(&self.context, value)?;
                        }

                        if let event::ApplicationEvent::Closed = value {
                            alive = false;
                        }
                    }

                    event::Event::InputDevice(value) => self.input.update_with(value),
                }
            }

            alive = alive && !self.context.is_shutdown();
            if !alive {
                break;
            }

            self.time.advance();
            self.graphics.swap_frames();

            let (video_info, duration) = {
                // Perform update and render submitting for frame [x], and drawing
                // frame [x-1] at the same time.
                task_sender.send(true).unwrap();

                // This will block the main-thread until all the graphics commands
                // is finished by GPU.
                let video_info = self.graphics.advance()?;
                let duration = join_receiver.recv().unwrap()?;
                (video_info, duration)
            };

            {
                let info = FrameInfo {
                    video: video_info,
                    duration: duration,
                    fps: self.time.shared().get_fps(),
                };

                let mut application = application.write().unwrap();
                application.on_post_update(&self.context, &info)?;
            }

            alive = alive && !self.context.is_shutdown() && !self.headless;
        }

        {
            let mut application = application.write().unwrap();
            application.on_exit(&self.context)?;
        }

        task_sender.send(false).unwrap();
        Ok(self)
    }

    fn main_thread<T>(
        receiver: mpsc::Receiver<bool>,
        sender: mpsc::Sender<Result<Duration>>,
        context: Arc<Context>,
        application: Arc<RwLock<T>>,
    ) where
        T: Application + Send + Sync + 'static,
    {
        thread::Builder::new()
            .name("LOGIC".into())
            .spawn(move || {
                //
                while receiver.recv().unwrap() {
                    sender
                        .send(Self::execute_frame(&context, &application))
                        .unwrap();
                }
            })
            .unwrap();
    }

    fn execute_frame<T>(ctx: &Context, application: &RwLock<T>) -> Result<Duration>
    where
        T: Application + Send + Sync + 'static,
    {
        let ts = Instant::now();

        let mut application = application.write().unwrap();
        application.on_update(ctx)?;
        application.on_render(ctx)?;

        Ok(Instant::now() - ts)
    }
}
