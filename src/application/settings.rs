//! Functions for loading game settings.

/// A structure containing configuration data for the game engine, which are
/// used to specify hardware setup stuff to create the window and other
/// context information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub engine: EngineSettings,
    pub window: WindowSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            max_fps: 0,
            max_inactive_fps: 0,
            time_smooth_step: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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