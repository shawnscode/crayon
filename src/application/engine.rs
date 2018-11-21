use std::sync::{Arc, Mutex};

use sched::prelude::LatchProbe;
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
    headless: bool,
}

struct EngineState {
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

impl Drop for EngineSystem {
    fn drop(&mut self) {
        crate::window::detach(self.events);

        unsafe {
            crate::res::discard();
            crate::input::discard();
            crate::video::discard();
            crate::window::discard();
            crate::sched::discard();
        }
    }
}

impl EngineSystem {
    /// Setup engine with specified settings.
    pub unsafe fn new(params: Params) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        crate::sched::setup(4, None, None);
        #[cfg(target_arch = "wasm32")]
        crate::sched::setup(0, None, None);

        crate::window::setup(params.window)?;
        crate::video::setup()?;
        crate::input::setup(params.input);
        crate::res::setup(params.res)?;

        let state = Arc::new(EngineState {
            alive: Mutex::new(true),
        });

        let sys = EngineSystem {
            events: crate::window::attach(state.clone()),
            state: state,
            headless: false,
        };

        Ok(sys)
    }

    pub unsafe fn new_headless(params: Params) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        crate::sched::setup(4, None, None);
        #[cfg(target_arch = "wasm32")]
        crate::sched::setup(0, None, None);

        crate::window::headless();
        crate::video::headless();
        crate::input::setup(params.input);
        crate::res::setup(params.res)?;

        let state = Arc::new(EngineState {
            alive: Mutex::new(false),
        });

        let sys = EngineSystem {
            events: crate::window::attach(state.clone()),
            state: state,
            headless: true,
        };

        Ok(sys)
    }

    #[inline]
    pub fn shutdown(&self) {
        *self.state.alive.lock().unwrap() = false;
    }

    #[inline]
    pub fn headless(&self) -> bool {
        self.headless
    }

    pub fn run_oneshot(&self) -> Result<()> {
        super::foreach(|v| v.on_pre_update())?;
        super::foreach(|v| v.on_update())?;
        super::foreach(|v| v.on_render())?;
        super::foreach_rev(|v| v.on_post_update())?;
        Ok(())
    }

    pub fn run<L, T, T2>(&self, latch: L, closure: T) -> Result<()>
    where
        L: LatchProbe + 'static,
        T: FnOnce() -> Result<T2> + 'static,
        T2: LifecycleListener + Send + 'static,
    {
        let state = self.state.clone();
        let mut closure = Some(closure);

        super::sys::run_forever(
            move || {
                super::foreach(|v| v.on_pre_update())?;
                super::foreach_rev(|v| v.on_post_update())?;
                Ok(!latch.is_set())
            },
            move || {
                let mut v = None;
                std::mem::swap(&mut closure, &mut v);

                let application = crate::application::attach(v.unwrap()()?);
                let state = state.clone();

                super::sys::run_forever(
                    move || {
                        super::foreach(|v| v.on_pre_update())?;
                        super::foreach(|v| v.on_update())?;
                        super::foreach(|v| v.on_render())?;
                        super::foreach_rev(|v| v.on_post_update())?;
                        Ok(*state.alive.lock().unwrap())
                    },
                    move || {
                        unsafe { crate::sched::terminate() };
                        crate::application::detach(application);
                        unsafe { super::late_discard() };
                        Ok(())
                    },
                )?;

                Ok(())
            },
        )?;

        Ok(())
    }
}
