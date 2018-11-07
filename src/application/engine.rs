use std::sync::{Arc, RwLock};

use super::*;

use input;
use utils::DoubleBuf;
use video;

type Result<T> = ::std::result::Result<T, ::failure::Error>;

#[derive(Default, Copy, Clone)]
struct ContextData {
    shutdown: bool,
}

/// The context of sub-systems that could be accessed from multi-thread environments safely.
#[derive(Clone)]
pub struct Context {
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
    pub time: time::TimeSystem,

    chan: Arc<DoubleBuf<Vec<Command>>>,

    pub(crate) context: Context,
    pub(crate) headless: bool,
}

#[derive(Debug, Clone)]
enum Command {
    ReceiveEvent(events::ApplicationEvent),
    Update,
    Render,
    PostUpdate,
    Exit,
}

impl Engine {
    /// Constructs a new, empty engine.
    pub fn new() -> Result<Self> {
        Engine::new_with(&settings::Settings::default())
    }

    /// Setup engine with specified settings.
    pub fn new_with(settings: &settings::Settings) -> Result<Self> {
        unsafe {
            ctx::setup();

            #[cfg(not(target_arch = "wasm32"))]
            crate::sched::setup(4, None, None);

            crate::res::setup();
        }

        let window = if settings.headless {
            window::Window::headless()
        } else {
            window::Window::new(settings.window.clone())?
        };

        let input = input::InputSystem::new(settings.input);
        let input_shared = input.shared();

        let video = if settings.headless {
            video::VideoSystem::headless()
        } else {
            video::VideoSystem::new(&window)?
        };

        let video_shared = video.shared();

        let time = time::TimeSystem::new(settings.engine);
        let time_shared = time.shared();

        // let mut ins = ins::InspectSystem::new();
        // ins.attach("Video", video_shared.clone());
        // ins.listen();

        let context = Context {
            input: input_shared,
            time: time_shared,
            video: video_shared,
            window: window.shared(),
            data: Arc::new(RwLock::new(ContextData::default())),
        };

        Ok(Engine {
            input: input,
            window: window,
            video: video,
            time: time,
            chan: Arc::new(DoubleBuf::default()),
            context: context,
            headless: settings.headless,
        })
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run<T: Application + Send + 'static>(mut self, application: T) -> Result<()> {
        let mut processor = Processor::new(&self, application);
        let (tx, rx) = ::std::sync::mpsc::channel();
        let (tx2, rx2) = ::std::sync::mpsc::channel();

        ::std::thread::Builder::new()
            .name("MainThread".into())
            .spawn(move || {
                while let Ok(_) = rx2.recv() {
                    if let Err(_) = tx.send(processor.advance()) {
                        break;
                    }
                }
            }).unwrap();

        self.chan.write().push(Command::Update);
        self.chan.write().push(Command::Render);
        self.chan.write().push(Command::PostUpdate);
        tx2.send(())?;

        while let Ok(Ok(_)) = rx.recv() {
            self.chan.swap();
            self.video.swap_frames();

            tx2.send(())?;
            if !self.advance(true)? {
                break;
            }
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn run<T: Application + Send + 'static>(mut self, application: T) -> Result<()> {
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let mut processor = Processor::new(&self, application);
        let window = web_sys::window().expect("should have a window in this context");
        let closure: Rc<RefCell<Option<Closure<FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let clone = closure.clone();

        *closure.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {
            self.chan.swap();
            self.video.swap_frames();

            processor.advance().unwrap();
            if self.advance(false).unwrap() {
                if let Some(inner) = clone.borrow().as_ref() {
                    let window = web_sys::window().expect("should have a window in this context");
                    window
                        .request_animation_frame(inner.as_ref().unchecked_ref())
                        .unwrap();
                }
            }
        })));

        if let Some(inner) = closure.borrow().as_ref() {
            window
                .request_animation_frame(inner.as_ref().unchecked_ref())
                .unwrap();
        }

        Ok(())
    }
}

impl Engine {
    fn advance(&mut self, schedule: bool) -> Result<bool> {
        let mut alive = true;
        let mut chan = self.chan.write();

        ctx::foreach(|v| v.on_update());

        self.time.advance(schedule);
        self.input.advance(self.window.hidpi());

        for v in self.window.advance() {
            match *v {
                events::Event::Application(value) => {
                    chan.push(Command::ReceiveEvent(value));

                    if let events::ApplicationEvent::Closed = value {
                        alive = false;
                    }
                }

                events::Event::InputDevice(value) => self.input.update_with(value),
            }
        }

        chan.push(Command::Update);
        chan.push(Command::Render);
        chan.push(Command::PostUpdate);

        if !alive || self.context.is_shutdown() || self.headless {
            chan.push(Command::Exit);
            return Ok(false);
        }

        self.video.advance(&self.window)?;
        self.window.swap_buffers()?;
        Ok(true)
    }
}

pub struct Processor<T: Application> {
    ctx: Context,
    chan: Arc<DoubleBuf<Vec<Command>>>,
    application: T,
}

impl<T: Application> Processor<T> {
    fn new(master: &Engine, application: T) -> Self {
        Processor {
            ctx: master.context.clone(),
            chan: master.chan.clone(),
            application: application,
        }
    }

    fn advance(&mut self) -> Result<()> {
        let mut cmds = self.chan.write_back_buf();
        for v in cmds.drain(..) {
            match v {
                Command::ReceiveEvent(evt) => {
                    self.application.on_receive_event(&self.ctx, evt)?;
                }

                Command::Update => {
                    self.application.on_update(&self.ctx)?;
                }

                Command::Render => {
                    self.application.on_render(&self.ctx)?;
                }

                Command::PostUpdate => {
                    self.application.on_post_update(&self.ctx)?;
                }

                Command::Exit => {
                    self.application.on_exit(&self.ctx)?;
                }
            }
        }

        Ok(())
    }
}
