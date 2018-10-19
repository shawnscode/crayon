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

pub mod events;
pub mod ins;
pub mod settings;
pub mod time;
pub mod window;

mod backends;
mod engine;

pub mod prelude {
    pub use super::engine::{Context, Engine};
    pub use super::settings::Settings;
    pub use super::window::{Window, WindowShared};
    pub use super::{Application, FrameInfo};
}

use std::time::Duration;

use errors::*;
use prelude::Context;
use video::VideoFrameInfo;

/// The collected information during last frame.
#[derive(Debug, Copy, Clone, Default)]
pub struct FrameInfo {
    pub video: VideoFrameInfo,
    pub duration: Duration,
    pub fps: u32,
}

/// `Application` is a user-friendly facade to build application, which consists of
/// several event functions that get executed in a pre-determined order.
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
    fn on_receive_event(&mut self, _: &Context, _: events::ApplicationEvent) -> Result<()> {
        Ok(())
    }

    /// `Application::on_exit` is called when exiting.
    fn on_exit(&mut self, _: &Context) -> Result<()> {
        Ok(())
    }
}
