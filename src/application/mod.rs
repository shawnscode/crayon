//! A unified application model across all target platforms.
//!
//! ## Application
//!
//! An application needs to run on all types of esoteric host platforms. To hide trivial
//! platform-specific details, we offers a convenient trait `Application` which defines
//! a simple application-state-model. While a state is active, the associated per-frame
//! methods are called in a pre-determined order.
//!
//! # Engine
//!
//! `Engine` is where we actully running the main loop and fire `Application` instance. It
//! also binds various essential systems in a central place.

pub mod errors;
pub mod settings;
pub mod event;
pub mod input;

pub use self::settings::Settings;
pub use self::event::{KeyboardButton, MouseButton};

mod engine;
pub use self::engine::Engine;

use std::sync::Arc;

use self::errors::*;
use graphics;
use resource;

pub struct FrameShared {
    pub video: Arc<graphics::GraphicsSystemShared>,
    pub resource: Arc<resource::ResourceSystemShared>,
}

pub struct FrameInfo {
    pub video: graphics::GraphicsFrameInfo,
}

/// `Application` is a user-friendly facade to building application, which defines a number
/// of event functions that get executed in a pre-determined order.
pub trait Application {
    /// `Application::on_update` is called every frame. Its the main workhorse
    /// function for frame updates.
    fn on_update(&mut self, _: &mut FrameShared) -> Result<()> {
        Ok(())
    }

    /// `Application::on_render` is called before we starts rendering the scene.
    fn on_render(&mut self, _: &mut FrameShared) -> Result<()> {
        Ok(())
    }

    /// `Application::on_post_update` is called after camera has rendered the scene.
    fn on_post_update(&mut self, _: &mut FrameShared, _: &FrameInfo) -> Result<()> {
        Ok(())
    }
}