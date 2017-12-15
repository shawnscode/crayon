//! Functions for loading game settings.

use input;

/// A structure containing configuration data for the game engine, which are
/// used to specify hardware setup stuff to create the window and other
/// context information.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub engine: EngineSettings,
    pub window: WindowSettings,
    pub input: InputSettings,
}

#[derive(Debug, Clone, Copy)]
pub struct EngineSettings {
    pub min_fps: u32,
    pub max_fps: u32,
    pub max_inactive_fps: u32,
    pub time_smooth_step: u32,
}

impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings {
            min_fps: 0,
            max_fps: 30,
            max_inactive_fps: 0,
            time_smooth_step: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowSettings {
    fn default() -> Self {
        WindowSettings {
            title: "Window".to_owned(),
            width: 640,
            height: 320,
        }
    }
}

pub type InputSettings = input::InputSetup;