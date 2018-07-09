use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use super::*;
use input;
use resource;
use video;

#[derive(Default, Copy, Clone)]
struct ContextData {
    shutdown: bool,
}

/// The context of sub-systems that could be accessed from multi-thread environments safely.
#[derive(Clone)]
pub struct Context {
    pub resource: Arc<resource::ResourceSystemShared>,
    pub input: Arc<input::InputSystemShared>,
    pub time: Arc<time::TimeSystemShared>,
    pub video: Arc<video::VideoSystemShared>,
    pub window: Arc<window::WindowShared>,

    data: Arc<RwLock<ContextData>>,
}

impl Context {
    /// Shutdown the whole application at the end of this frame.
    pub fn shutdown(&self) {
        self.data.write().unwrap().shutdown = true;
    }

    /// Returns true if we are going to shutdown the application at the end of this frame.
    pub fn is_shutdown(&self) -> bool {
        self.data.read().unwrap().shutdown
    }
}

/// `Engine` is the root object of the game application. It binds various sub-systems in
/// a central place and takes take of trivial tasks like the execution order or life-time
/// management.
pub struct Engine {
    pub events_loop: event::EventsLoop,
    pub window: window::Window,
    pub input: input::InputSystem,
    pub video: video::VideoSystem,
    pub resource: resource::ResourceSystem,
    pub time: time::TimeSystem,

    context: Context,
    headless: bool,
}

impl Engine {
    /// Constructs a new, empty engine.
    pub fn new() -> Result<Self> {
        Engine::new_with(&Settings::default())
    }

    /// Setup engine with specified settings.
    pub fn new_with(settings: &Settings) -> Result<Self> {
        let input = input::InputSystem::new(settings.input);
        let input_shared = input.shared();

        let events_loop = event::EventsLoop::new();
        let window = window::Window::new(settings.window.clone(), events_loop.underlaying())?;

        let resource = resource::ResourceSystem::new()?;
        let resource_shared = resource.shared();

        let video = video::VideoSystem::new(&window, resource_shared.clone())?;
        let video_shared = video.shared();

        let time = time::TimeSystem::new(settings.engine)?;
        let time_shared = time.shared();

        let context = Context {
            resource: resource_shared,
            input: input_shared,
            time: time_shared,
            video: video_shared,
            window: window.shared(),
            data: Arc::new(RwLock::new(ContextData::default())),
        };

        Ok(Engine {
            events_loop: events_loop,
            input: input,
            window: window,
            video: video,
            resource: resource,
            time: time,

            context: context,
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
            self.window.advance();
            self.input.advance(self.window.hidpi());

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
            self.video.swap_frames();

            let (video_info, duration) = {
                // Perform update and render submitting for frame [x], and drawing
                // frame [x-1] at the same time.
                task_sender.send(true).unwrap();

                // This will block the main-thread until all the video commands
                // is finished by GPU.
                let video_info = self.video.advance(&self.window)?;
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
        context: Context,
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
