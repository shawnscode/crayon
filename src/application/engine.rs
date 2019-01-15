use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::sched::prelude::LatchProbe;
use crate::window::prelude::{Event, EventListener, EventListenerHandle, WindowEvent};

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
    alive: AtomicBool,
}

impl EventListener for Arc<EngineState> {
    fn on(&mut self, v: &Event) -> Result<()> {
        if let Event::Window(WindowEvent::Closed) = *v {
            self.alive.store(false, Ordering::Relaxed);
        }

        Ok(())
    }
}

impl Drop for EngineSystem {
    fn drop(&mut self) {
        crate::window::detach(self.events);

        unsafe {
            crate::res::inside::discard();
            crate::input::inside::discard();
            crate::video::inside::discard();
            crate::window::inside::discard();
            crate::sched::inside::discard();
        }
    }
}

impl EngineSystem {
    /// Setup engine with specified settings.
    pub unsafe fn new(params: Params) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        crate::sched::inside::setup(4, None, None);
        #[cfg(target_arch = "wasm32")]
        crate::sched::inside::setup(0, None, None);

        crate::window::inside::setup(params.window)?;
        crate::video::inside::setup()?;
        crate::input::inside::setup(params.input);
        crate::res::inside::setup(params.res)?;

        let state = Arc::new(EngineState {
            alive: AtomicBool::new(true),
        });

        let sys = EngineSystem {
            events: crate::window::attach(state.clone()),
            state,
            headless: false,
        };

        Ok(sys)
    }

    pub unsafe fn new_headless(params: Params) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        crate::sched::inside::setup(4, None, None);
        #[cfg(target_arch = "wasm32")]
        crate::sched::inside::setup(0, None, None);

        crate::window::inside::headless();
        crate::video::inside::headless();
        crate::input::inside::setup(params.input);
        crate::res::inside::setup(params.res)?;

        let state = Arc::new(EngineState {
            alive: AtomicBool::new(false),
        });

        let sys = EngineSystem {
            events: crate::window::attach(state.clone()),
            state,
            headless: true,
        };

        Ok(sys)
    }

    #[inline]
    pub fn shutdown(&self) {
        self.state.alive.store(false, Ordering::Relaxed);
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

                        Ok(state.alive.load(Ordering::Relaxed))
                    },
                    move || {
                        unsafe { crate::sched::inside::terminate() };
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
