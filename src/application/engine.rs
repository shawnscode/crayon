use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::*;

use input;
use res;
use sched;
use video;

type Result<T> = ::std::result::Result<T, ::failure::Error>;

#[derive(Default, Copy, Clone)]
struct ContextData {
    shutdown: bool,
}

/// The context of sub-systems that could be accessed from multi-thread environments safely.
#[derive(Clone)]
pub struct Context {
    pub res: Arc<res::ResourceSystemShared>,
    // pub sched: Arc<sched::ScheduleSystemShared>,
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
    pub window: window::Window,
    pub input: input::InputSystem,
    pub video: video::VideoSystem,
    pub res: res::ResourceSystem,
    pub time: time::TimeSystem,
    // pub sched: sched::ScheduleSystem,
    pub(crate) context: Context,
    pub(crate) headless: bool,
}

impl Engine {
    /// Constructs a new, empty engine.
    pub fn new() -> Result<Self> {
        Engine::new_with(&settings::Settings::default())
    }

    /// Setup engine with specified settings.
    pub fn new_with(settings: &settings::Settings) -> Result<Self> {
        let window = if settings.headless {
            window::Window::headless()
        } else {
            window::Window::new(settings.window.clone())?
        };

        let input = input::InputSystem::new(settings.input);
        let input_shared = input.shared();

        // let sched = sched::ScheduleSystem::new(6, None, None);
        // let sched_shared = sched.shared();

        let res = res::ResourceSystem::new()?;
        let res_shared = res.shared();

        let video = if settings.headless {
            video::VideoSystem::headless(res_shared.clone())
        } else {
            video::VideoSystem::new(&window, res_shared.clone())?
        };

        let video_shared = video.shared();

        let time = time::TimeSystem::new(settings.engine);
        let time_shared = time.shared();

        // let mut ins = ins::InspectSystem::new();
        // ins.attach("Video", video_shared.clone());
        // ins.listen();

        let context = Context {
            res: res_shared,
            input: input_shared,
            time: time_shared,
            video: video_shared,
            window: window.shared(),
            // sched: sched_shared,
            data: Arc::new(RwLock::new(ContextData::default())),
        };

        Ok(Engine {
            input: input,
            window: window,
            video: video,
            res: res,
            time: time,
            // sched: sched,
            context: context,
            headless: settings.headless,
        })
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    #[cfg(target_arch = "wasm32")]
    pub fn run_wasm<T>(self, application: T) -> Result<()>
    where
        T: Application + Send + 'static,
    {
        let wasm = crate::sys::WasmEngine::new(self, application);
        wasm.borrow_mut().run()?;
        Ok(())
    }

    /// Run the main loop of `Engine`, this will block the working
    /// thread until we finished.
    pub fn run<T>(mut self, application: T) -> Result<Self>
    where
        T: Application + Send + 'static,
    {
        // let dir = ::std::env::current_dir()?;
        // info!("CWD: {:?}.", dir);

        let latch = Arc::new(sched::latch::LockLatch::new());
        Self::execute_frame(latch.clone(), self.context.clone(), application);

        let mut alive = true;
        let mut frame_info = None;
        while alive {
            self.time.advance(true);

            let (mut application, duration) = latch.wait_and_take()?;
            if let Some(frame_info) = frame_info {
                application.on_post_update(&self.context, &frame_info)?;
            }

            self.input.advance(self.window.hidpi());
            // Poll any possible events first.
            for v in self.window.advance() {
                match *v {
                    events::Event::Application(value) => {
                        application.on_receive_event(&self.context, value)?;

                        if let events::ApplicationEvent::Closed = value {
                            alive = false;
                        }
                    }

                    events::Event::InputDevice(value) => self.input.update_with(value),
                }
            }

            alive = alive && !self.context.is_shutdown() && !self.headless;
            if !alive {
                application.on_exit(&self.context)?;
                break;
            }

            // Perform update and render submitting for frame [x], and drawing
            // frame [x-1] at the same time.
            self.video.swap_frames();
            Self::execute_frame(latch.clone(), self.context.clone(), application);

            // This will block the main-thread until all the video commands is finished by GPU.
            let video_info = self.video.advance(&self.window)?;
            self.window.swap_buffers()?;

            //
            frame_info = Some(FrameInfo {
                video: video_info,
                duration: duration,
                fps: self.time.shared().fps(),
            });
        }

        // self.sched.terminate();
        Ok(self)
    }

    fn execute_frame<T>(
        latch: Arc<sched::latch::LockLatch<Result<(T, Duration)>>>,
        ctx: Context,
        mut application: T,
    ) where
        T: Application + Send + 'static,
    {
        let ctx_clone = ctx.clone();

        let run = move || {
            let ts = crate::sys::instant();
            application.on_update(&ctx_clone)?;
            application.on_render(&ctx_clone)?;
            Ok((application, crate::sys::instant() - ts))
        };

        latch.set(run());

        // ctx.sched.spawn(move || latch.set(run()));
    }
}
