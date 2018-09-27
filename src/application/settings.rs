//! Functions for loading game settings.

use input;
use math::prelude::Vector2;

/// A structure containing configuration data for the game engine, which are
/// used to specify hardware setup stuff to create the window and other
/// context information.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub engine: EngineParams,
    pub window: WindowParams,
    pub input: input::InputParams,
    pub headless: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct EngineParams {
    pub min_fps: u32,
    pub max_fps: u32,
    pub max_inactive_fps: u32,
    pub time_smooth_step: u32,
}

impl Default for EngineParams {
    fn default() -> Self {
        EngineParams {
            min_fps: 0,
            max_fps: 30,
            max_inactive_fps: 0,
            time_smooth_step: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowParams {
    /// Sets the title of window.
    pub title: String,
    /// Sets the size in *points* of the client area of the window.
    pub size: Vector2<u32>,
    /// Sets the multisampling level to request. A value of 0 indicates that
    /// multisampling must not be enabled.
    pub multisample: u16,
    /// Specifies whether should we have vsync.
    pub vsync: bool,
}

impl Default for WindowParams {
    fn default() -> Self {
        WindowParams {
            title: "Window".to_owned(),
            size: Vector2::new(640, 320),
            multisample: 2,
            vsync: false,
        }
    }
}
