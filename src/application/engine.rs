use std::sync::{Arc, Mutex};

use window::prelude::{Event, EventListener, EventListenerHandle, WindowEvent};

use super::lifecycle::LifecycleListener;
use super::Params;

type Result<T> = ::std::result::Result<T, ::failure::Error>;

/// `Engine` is the root object of the game application. It binds various sub-systems in
/// a central place and takes take of trivial tasks like the execution order or life-time
/// management.
pub struct EngineSystem {
    events: EventListenerHandle,
    state: Arc<EngineState>,
}

struct EngineState {
    headless: bool,
    alive: Mutex<bool>,
}

impl EventListener for Arc<EngineState> {
    fn on(&mut self, v: &Event) -> Result<()> {
        if let &Event::Window(WindowEvent::Closed) = v {
            *self.alive.lock().unwrap() = false;
        }

        Ok(())
    }
}

impl EngineState {
    fn advance(&self) -> Result<bool> {
        super::foreach(|v| v.on_pre_update())?;
        super::foreach(|v| v.on_update())?;
        super::foreach(|v| v.on_render())?;
        super::foreach_rev(|v| v.on_post_update())?;

        Ok(*self.alive.lock().unwrap() && !self.headless)
    }
}

impl Drop for EngineSystem {
    fn drop(&mut self) {
        crate::window::detach(self.events);

        unsafe {
            crate::input::discard();
            crate::video::discard();
            crate::window::discard();
            crate::res::discard();
            crate::sched::discard();
        }
    }
}

impl EngineSystem {
    /// Setup engine with specified settings.
    pub fn new(params: Params) -> Result<Self> {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            crate::sched::setup(4, None, None);
            #[cfg(target_arch = "wasm32")]
            crate::sched::setup(0, None, None);

            if !params.headless {
                crate::window::setup(params.window)?;
                crate::video::setup()?;
            } else {
                crate::window::headless();
                crate::video::headless();
            }

            crate::input::setup(params.input);
            crate::res::setup(params.res)?;

            let state = Arc::new(EngineState {
                headless: params.headless,
                alive: Mutex::new(true),
            });

            let sys = EngineSystem {
                events: crate::window::attach(state.clone()),
                state: state,
            };

            Ok(sys)
        }
    }

    pub fn shutdown(&self) {
        *self.state.alive.lock().unwrap() = false;
    }

    pub fn run<T>(&self, application: T) -> Result<()>
    where
        T: LifecycleListener + Send + 'static,
    {
        let application = crate::application::attach(application);
        let state = self.state.clone();

        super::sys::run_forever(move || {
            let rsp = state.advance();

            match rsp {
                Ok(true) => {}
                _ => {
                    crate::application::detach(application);
                    unsafe { super::late_discard() };
                }
            };

            rsp
        })?;

        Ok(())
    }
}
