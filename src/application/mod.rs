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
//! ``` no_run
//! struct Window { ... }
//! impl Application for Window { ... }
//!
//! fn main() {
//!     let mut engine = Engine::new();
//!     let window = Window::new(&mut engine).unwrap();
//!     engine.run(window).unrwap();
//! }
//! ```
//!
//! # Engine
//!
//! `Engine` mentioned above is the most fundamental module in crayon. It binds various
//! essential systems in a central place, and responsible for running the main loop.
//!

pub mod errors;
pub mod settings;
pub mod context;
pub mod event;

pub mod time;
pub use self::time::TimeSystem;

pub use self::settings::Settings;
pub use self::context::Context;

mod engine;
pub use self::engine::Engine;

use self::errors::*;
use graphics::GraphicsFrameInfo;
use std::time::Duration;

/// The collected information during last frame.
#[derive(Debug, Copy, Clone, Default)]
pub struct FrameInfo {
    pub video: GraphicsFrameInfo,
    pub duration: Duration,
    pub fps: u32,
}

/// `Application` is a user-friendly facade to building application, which defines a number
/// of event functions that get executed in a pre-determined order.
pub trait Application {
    /// `Application::on_update` is called every frame. Its the main workhorse
    /// function for frame updates.
    fn on_update(&mut self, _: &Context) -> Result<()> {
        Ok(())
    }

    /// `Application::on_render` is called before we starts rendering the scene.
    fn on_render(&mut self, _: &Context) -> Result<()> {
        Ok(())
    }

    /// `Application::on_post_update` is called after camera has rendered the scene.
    fn on_post_update(&mut self, _: &Context, _: &FrameInfo) -> Result<()> {
        Ok(())
    }

    /// `Application::on_update` is called when receiving application event.
    fn on_receive_event(&mut self, _: &Context, _: event::ApplicationEvent) -> Result<()> {
        Ok(())
    }

    /// `Application::on_exit` is called when exiting.
    fn on_exit(&mut self, _: &Context) -> Result<()> {
        Ok(())
    }
}