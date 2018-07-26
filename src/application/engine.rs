use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use super::*;
use input;
use res;
use sched;
use video;

#[derive(Default, Copy, Clone)]
struct ContextData {
    shutdown: bool,
}

/// The context of sub-systems that could be accessed from multi-thread environments safely.
#[derive(Clone)]
pub struct Context {
    pub res: Arc<res::ResourceSystemShared>,
    pub input: Arc<input::InputSystemShared>,
    pub time: Arc<time::TimeSystemShared>,
    pub video: Arc<video::VideoSystemShared>,
    pub window: Arc<window::WindowShared>,
    pub sched: Arc<sched::ScheduleSystemShared>,

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
    pub window: window::Window,
    pub input: input::InputSystem,
    pub video: video::VideoSystem,
    pub res: res::ResourceSystem,
    pub time: time::TimeSystem,
    pub sched: sched::ScheduleSystem,

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
        let sched = sched::ScheduleSystem::new(6, None, None);
        let sched_shared = sched.shared();

        let input = input::InputSystem::new(settings.input);
        let input_shared = input.shared();

        let window = window::Window::new(settings.window.clone())?;

        let res = res::ResourceSystem::new(sched_shared.clone())?;
        let res_shared = res.shared();

        let video = video::VideoSystem::new(&window)?;
        let video_shared = video.shared();

        let time = time::TimeSystem::new(settings.engine)?;
        let time_shared = time.shared();

        res.register(video::assets::texture_loader::TextureLoader::new(
            video_shared.clone(),
        ));

        let context = Context {
            res: res_shared,
            input: input_shared,
            time: time_shared,
            video: video_shared,
            window: window.shared(),
            sched: sched_shared,
            data: Arc::new(RwLock::new(ContextData::default())),
        };

        Ok(Engine {
            input: input,
            window: window,
            video: video,
            res: res,
            time: time,
            sched: sched,

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
        println!("CWD: {:?}.", dir);

        let (task_sender, task_receiver) = mpsc::channel();
        let (join_sender, join_receiver) = mpsc::channel();
        Self::main_thread(
            task_receiver,
            join_sender,
            self.context.clone(),
            application.clone(),
        );
        task_sender.send(true).unwrap();

        let mut alive = true;
        while alive {
            self.input.advance(self.window.hidpi());

            // Poll any possible events first.
            for v in self.window.advance() {
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

            self.res.advance();
            self.time.advance();
            self.video.swap_frames();

            let (video_info, duration) = {
                let duration = join_receiver.recv().unwrap()?;

                // Perform update and render submitting for frame [x], and drawing
                // frame [x-1] at the same time.
                task_sender.send(true).unwrap();
                // This will block the main-thread until all the video commands is finished by GPU.
                let video_info = self.video.advance(&self.window)?;
                (video_info, duration)
            };

            self.window.swap_buffers()?;

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

        self.sched.terminate();
        self.sched.wait_until_terminated();
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
