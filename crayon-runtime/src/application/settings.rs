//! Functions for loading game settings.

use std::path::Path;
use std::fs::File;
use std::io::Read;

use bincode;

use super::errors::*;

/// A structure containing configuration data for the game engine, which are
/// used to specify hardware setup stuff to create the window and other
/// context information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub engine: EngineSettings,
    pub window: WindowSettings,
}

impl Settings {
    /// Create application settings from data at path.
    pub fn load_from<P>(path: P) -> Result<Self>
        where P: AsRef<Path>
    {
        let mut buf = Vec::new();
        let mut file = File::open(&path)?;
        file.read_to_end(&mut buf)?;

        Ok(bincode::deserialize(&buf)?)
    }
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