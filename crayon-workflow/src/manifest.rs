use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;

use toml;

use errors::*;
use resource::Resource;
use crayon::core::settings::Settings;
use serialization;

/// Workflow manifest of crayon project.
#[derive(Debug, Clone)]
pub struct Manifest {
    dir: PathBuf,
    workspace: PathBuf,

    pub resources: Vec<PathBuf>,
    pub types: HashMap<String, Resource>,
    pub settings: Settings,
}

impl Manifest {
    pub fn find<P>(path: P) -> Result<Manifest>
        where P: AsRef<Path>
    {
        if let Ok(dir) = fs::metadata(&path) {
            if dir.is_dir() {
                let file_path = path.as_ref().join("Crayon.toml");
                if let Ok(file) = fs::metadata(&file_path) {
                    if file.is_file() {
                        return Manifest::parse(&file_path);
                    }
                } else {
                    if let Some(parent) = path.as_ref().parent() {
                        return Manifest::find(parent);
                    }
                }
            }
        }

        bail!("Failed to find manifest Crayon.toml.");
    }

    pub fn load_from<P>(path: P) -> Result<Manifest>
        where P: AsRef<Path>
    {
        if let Ok(file) = fs::metadata(path.as_ref()) {
            if file.is_file() {
                return Manifest::parse(path.as_ref());
            }
        }

        bail!("Failed to parse manifest at {:?}.", path.as_ref());
    }

    /// Setup workspace.
    pub fn setup(self) -> Result<Self> {
        if !self.workspace.exists() {
            fs::create_dir_all(&self.workspace)?;
        }

        Ok(self)
    }

    /// The directory of workspace.
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    /// The directory where this manifest locates.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Save settings as serialization data.
    pub fn save_settings<P>(&self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        serialization::serialize(&self.settings, path, false)?;
        Ok(())
    }

    fn parse(path: &Path) -> Result<Manifest> {
        if let Ok(mut file) = fs::File::open(path) {
            let mut raw = String::new();
            file.read_to_string(&mut raw)?;

            let value: toml::Value = raw.parse()?;
            let dir = path.parent().unwrap().to_owned();
            let absolute_dir = if dir.is_relative() {
                let mut wd = ::std::env::current_dir()?;
                wd.push(dir);
                wd
            } else {
                dir
            };

            let mut manifest = Manifest {
                workspace: absolute_dir.join(".crayon"),
                dir: absolute_dir,
                settings: Settings::default(),
                resources: Vec::new(),
                types: HashMap::new(),
            };

            if let Some(runtime) = value.get("Runtime").and_then(|v| v.as_table()) {
                if let Some(engine_settings) =
                    runtime.get("EngineSettings").and_then(|v| v.as_table()) {

                    if let Some(v) = engine_settings.get("min_fps").and_then(|v| v.as_integer()) {
                        manifest.settings.engine.min_fps = v as u32;
                    }

                    if let Some(v) = engine_settings.get("max_fps").and_then(|v| v.as_integer()) {
                        manifest.settings.engine.max_fps = v as u32;
                    }

                    if let Some(v) = engine_settings
                           .get("time_smooth_step")
                           .and_then(|v| v.as_integer()) {
                        manifest.settings.engine.time_smooth_step = v as u32;
                    }
                }

                if let Some(window_settings) =
                    runtime.get("WindowSettings").and_then(|v| v.as_table()) {
                    if let Some(v) = window_settings.get("width").and_then(|v| v.as_integer()) {
                        manifest.settings.window.width = v as u32;
                    }

                    if let Some(v) = window_settings.get("height").and_then(|v| v.as_integer()) {
                        manifest.settings.window.height = v as u32;
                    }

                    if let Some(v) = window_settings.get("title").and_then(|v| v.as_str()) {
                        manifest.settings.window.title = v.to_owned();
                    }
                }
            }

            if let Some(workflow) = value.get("Workflow").and_then(|v| v.as_table()) {
                if let Some(project_settings) =
                    workflow.get("ProjectSettings").and_then(|v| v.as_table()) {

                    if let Some(resources) = project_settings.get("resources").and_then(|v| {
                        v.as_array()
                    }) {
                        for item in resources {
                            if let Some(v) = item.as_str() {
                                let dir = manifest.dir.join(v);
                                if let Ok(true) = fs::metadata(&dir).and_then(|v| Ok(v.is_dir())) {
                                    manifest.resources.push(dir);
                                }
                            }
                        }
                    }
                }

                if let Some(import_settings) =
                    workflow.get("ImportSettings").and_then(|v| v.as_table()) {

                    if let Some(types) = import_settings.get("bytes").and_then(|v| v.as_array()) {
                        for item in types {
                            if let Some(v) = item.as_str() {
                                manifest
                                    .types
                                    .insert(v.trim_matches('.').to_owned(), Resource::Bytes);
                            }
                        }
                    }

                    if let Some(types) =
                        import_settings.get("textures").and_then(|v| v.as_array()) {
                        for item in types {
                            if let Some(v) = item.as_str() {
                                manifest
                                    .types
                                    .insert(v.trim_matches('.').to_owned(), Resource::Texture);
                            }
                        }
                    }

                    if let Some(types) = import_settings.get("atlases").and_then(|v| v.as_array()) {
                        for item in types {
                            if let Some(v) = item.as_str() {
                                manifest
                                    .types
                                    .insert(v.trim_matches('.').to_owned(), Resource::Atlas);
                            }
                        }
                    }

                    if let Some(types) = import_settings.get("shaders").and_then(|v| v.as_array()) {
                        for item in types {
                            if let Some(v) = item.as_str() {
                                manifest
                                    .types
                                    .insert(v.trim_matches('.').to_owned(), Resource::Shader);
                            }
                        }
                    }
                }
            }

            Ok(manifest)
        } else {
            bail!("Crayon.toml at {:?} is not valid.", path);
        }
    }
}