//! A unified application model across all target platforms.
//!
//! # Application
//!
//! An application needs to run on all types of esoteric host platforms. To hide trivial
//! platform-specific details, we offers a convenient trait `Application` facade which
//! defined methods, that will be called in a pre-determined order every frame.
//!
//! The most intuitive and simple setup function could be something like:
//!
//! ```rust,ignore
//! use crayon::application::prelude::*;
//!
//! struct Window {}
//! impl Application for Window {}
//!
//! fn main() {
//!     let window = Window {};
//!     Engine::new().unwrap().run(window).unwrap();
//! }
//! ```
//!
//! # Engine
//!
//! `Engine` mentioned above is the most fundamental module in crayon. It binds various
//! essential systems in a central place, and responsible for running the main loop.
//!

pub mod ins;
pub mod sys;

mod engine;
mod lifecycle;
mod time;

pub mod prelude {
    pub use super::lifecycle::{LifecycleListener, LifecycleListenerHandle};
    pub use super::Application;
    pub use super::Params;
}

use errors::*;

use self::lifecycle::{LifecycleListener, LifecycleListenerHandle};
use window::events::WindowEvent;

/// `Application` is a user-friendly facade to build application, which consists of
/// several event functions that get executed in a pre-determined order.
pub trait Application {
    /// `Application::on_update` is called every frame. Its the main workhorse
    /// function for frame updates.
    fn on_update(&mut self) -> Result<()> {
        Ok(())
    }

    /// `Application::on_render` is called before we starts rendering the scene.
    fn on_render(&mut self) -> Result<()> {
        Ok(())
    }

    /// `Application::on_post_update` is called after camera has rendered the scene.
    fn on_post_update(&mut self) -> Result<()> {
        Ok(())
    }

    /// `Application::on_update` is called when receiving application event.
    fn on_receive_event(&mut self, _: WindowEvent) -> Result<()> {
        Ok(())
    }

    /// `Application::on_exit` is called when exiting.
    fn on_exit(&mut self) -> Result<()> {
        Ok(())
    }
}

use self::engine::EngineSystem;
use self::inside::{ctx, lifecycle_ctx, time_ctx, CTX, LIFECYCLE_CTX, TIME_CTX};
use self::lifecycle::LifecycleSystem;
use self::time::TimeSystem;

use input::InputParams;
use res::ResourceParams;
use window::WindowParams;

/// A structure containing configuration data for the game engine, which are
/// used to specify hardware setup stuff to create the window and other
/// context information.
#[derive(Debug, Clone)]
pub struct Params {
    /// Is this a headless running?
    pub headless: bool,
    /// Set minimum frames per second. If fps goes lower than this, time will
    /// appear to slow. This is useful for some subsystems required strict minimum
    /// time step per frame, such like Collision checks.
    pub min_fps: u32,
    /// Set maximum frames per second. The engine will sleep if fps is higher
    /// than this for less resource(e.g. power) consumptions.
    pub max_fps: u32,
    /// Set maximum frames per second when the application does not have input
    /// focus.
    pub max_inactive_fps: u32,
    /// Set how many frames to average for timestep smoothing.
    pub time_smooth_step: u32,
    /// The setup parameters for window sub-system.
    pub window: WindowParams,
    /// The setup parameters for input sub-system.
    pub input: InputParams,
    /// The setup params for resource sub-system.
    pub res: ResourceParams,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            min_fps: 0,
            max_fps: 30,
            max_inactive_fps: 0,
            time_smooth_step: 0,
            headless: false,
            window: WindowParams::default(),
            input: InputParams::default(),
            res: ResourceParams::default(),
        }
    }
}

impl Params {
    /// Create a headless `Params`.
    pub fn headless() -> Self {
        let mut params = Params::default();
        params.headless = true;
        params
    }

    #[allow(unused_mut)]
    pub fn validate(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if self.max_fps > 0 {
                self.max_fps = 0;
                warn!("The max FPS could be not controlled in web environment.");
            }
        }
    }
}

/// Setup the core system.
pub fn setup(mut params: Params) -> Result<()> {
    unsafe {
        debug_assert!(LIFECYCLE_CTX.is_null(), "duplicated setup of crayon.");

        sys::init();
        params.validate();

        LIFECYCLE_CTX = Box::into_raw(Box::new(LifecycleSystem::new()));
        TIME_CTX = Box::into_raw(Box::new(TimeSystem::new(&params)));
        CTX = Box::into_raw(Box::new(EngineSystem::new(params)?));

        Ok(())
    }
}

#[inline]
pub fn run<T: LifecycleListener + Send + 'static>(application: T) -> Result<()> {
    ctx().run(application)
}

/// Discard the core system.
#[inline]
pub fn discard() {
    ctx().shutdown()
}

pub(crate) unsafe fn late_discard() {
    drop(Box::from_raw(CTX as *mut EngineSystem));
    CTX = 0 as *const EngineSystem;

    drop(Box::from_raw(TIME_CTX as *mut TimeSystem));
    TIME_CTX = 0 as *const TimeSystem;

    drop(Box::from_raw(CTX as *mut LifecycleSystem));
    LIFECYCLE_CTX = 0 as *const LifecycleSystem;
}

/// Checks if the engine is enabled.
#[inline]
pub fn valid() -> bool {
    unsafe { !LIFECYCLE_CTX.is_null() }
}

#[inline]
pub fn attach<T>(lis: T) -> LifecycleListenerHandle
where
    T: LifecycleListener + 'static,
{
    lifecycle_ctx().attach(lis)
}

#[inline]
pub fn detach(handle: LifecycleListenerHandle) {
    lifecycle_ctx().detach(handle)
}

/// Set minimum frames per second. If fps goes lower than this, time will
/// appear to slow. This is useful for some subsystems required strict minimum
/// time step per frame, such like Collision checks.
#[inline]
pub fn set_min_fps(fps: u32) {
    time_ctx().set_min_fps(fps);
}

/// Set maximum frames per second. The Time will sleep if fps is higher
/// than this for less resource(e.g. power) consumptions.
#[allow(unused_assignments, unused_mut)]
#[inline]
pub fn set_max_fps(mut fps: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        warn!("The max FPS could be not controlled in web environment.");
        fps = 0;
    }

    time_ctx().set_max_fps(fps);
}

/// Set maximum frames per second when the application does not have input
/// focus.
#[inline]
pub fn set_max_inactive_fps(fps: u32) {
    time_ctx().set_max_inactive_fps(fps);
}

/// Set how many frames to average for timestep smoothing.
#[inline]
pub fn set_time_smoothing_step(step: u32) {
    time_ctx().set_time_smoothing_step(step);
}

/// Gets current fps.
#[inline]
pub fn fps() -> u32 {
    time_ctx().fps()
}

/// Gets the duration duraing last frame.
#[inline]
pub fn frame_duration() -> ::std::time::Duration {
    time_ctx().frame_duration()
}

#[inline]
fn foreach<T>(func: T) -> Result<()>
where
    T: Fn(&mut dyn LifecycleListener) -> Result<()>,
{
    lifecycle_ctx().foreach(func)
}

#[inline]
fn foreach_rev<T>(func: T) -> Result<()>
where
    T: Fn(&mut dyn LifecycleListener) -> Result<()>,
{
    lifecycle_ctx().foreach_rev(func)
}

mod inside {
    use super::engine::EngineSystem;
    use super::lifecycle::LifecycleSystem;
    use super::time::TimeSystem;

    pub static mut LIFECYCLE_CTX: *const LifecycleSystem = 0 as *const LifecycleSystem;
    pub static mut TIME_CTX: *const TimeSystem = 0 as *const TimeSystem;
    pub static mut CTX: *const EngineSystem = 0 as *const EngineSystem;

    pub fn lifecycle_ctx() -> &'static LifecycleSystem {
        unsafe {
            debug_assert!(
                !LIFECYCLE_CTX.is_null(),
                "lifecycle system has not been initialized properly."
            );

            &*LIFECYCLE_CTX
        }
    }

    pub fn time_ctx() -> &'static TimeSystem {
        unsafe {
            debug_assert!(
                !TIME_CTX.is_null(),
                "time system has not been initialized properly."
            );

            &*TIME_CTX
        }
    }

    pub fn ctx() -> &'static EngineSystem {
        unsafe {
            debug_assert!(
                !TIME_CTX.is_null(),
                "engine has not been initialized properly."
            );

            &*CTX
        }
    }
}
