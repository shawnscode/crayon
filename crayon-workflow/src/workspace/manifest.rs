use std::{fs, env};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use toml;

use crayon::core::settings::Settings as RuntimeSettings;
use resource::ResourceType;
use utils::toml::*;
use utils::bincode;
use errors::*;

/// `WorkspaceSettings`
#[derive(Debug, Clone)]
pub struct WorkspaceSettings {
    pub resource_folders: Vec<PathBuf>,
    pub resource_exts: HashMap<String, ResourceType>,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        WorkspaceSettings {
            resource_folders: Vec::new(),
            resource_exts: HashMap::new(),
        }
    }
}

/// For every projects based on crayon, we will have a editable `manifest` file in the
/// root folder. User could configurate the wanted workflow and runtime behaviours through
/// editing the toml file `workspace.toml`.
#[derive(Debug, Clone)]
pub struct Manifest {
    /// The directory where we load this manifest from, and also indicates the root
    /// folder of the project's `Workspace`.
    dir: PathBuf,

    /// The settings of workspace.
    workspace: WorkspaceSettings,

    /// The settings of runtime.
    runtime: RuntimeSettings,
}

impl Manifest {
    /// Load toml based `Manifest` at path.
    pub fn load_from<P>(path: P) -> Result<Manifest>
        where P: AsRef<Path>
    {
        let value: toml::Value = {
            let mut file = fs::File::open(path.as_ref())?;
            let mut payload = String::new();
            file.read_to_string(&mut payload)?;
            payload.parse()?
        };

        let absolute_dir = {
            let dir = path.as_ref().parent().unwrap().to_owned();
            if dir.is_relative() {
                let mut wd = env::current_dir()?;
                wd.push(dir);
                wd
            } else {
                dir
            }
        };

        // Parse runtime setup settings.
        let mut runtime = RuntimeSettings::default();

        if let Some(value) = load(&value, &["Runtime", "EngineSettings"]) {
            runtime.engine.min_fps = load_as_u32(&value, &["min_fps"]).unwrap_or(0);
            runtime.engine.max_fps = load_as_u32(&value, &["max_fps"]).unwrap_or(0);
            runtime.engine.time_smooth_step = load_as_u32(&value, &["time_smooth_step"])
                .unwrap_or(0);
        }

        if let Some(value) = load(&value, &["Runtime", "WindowSettings"]) {
            runtime.window.width = load_as_u32(&value, &["width"]).unwrap_or(640);
            runtime.window.height = load_as_u32(&value, &["height"]).unwrap_or(480);
            runtime.window.title = load_as_str(&value, &["title"])
                .unwrap_or("Window")
                .to_owned();
        }

        // Parse workflow settings.
        let mut workspace = WorkspaceSettings::default();

        if let Some(value) = load(&value, &["Workflow", "ProjectSettings"]) {
            if let Some(folders) = load_as_array(&value, &["resources"]).and_then(|v| {
                Some(v.iter()
                         .filter_map(|v| v.as_str())
                         .filter_map(|v| absolute_dir.join(v).canonicalize().ok())
                         .filter(|v| v.exists() && v.is_dir())
                         .collect())
            }) {
                workspace.resource_folders = folders;
            }
        }

        if let Some(value) = load(&value, &["Workflow", "ImportSettings"]) {
            workspace.resource_exts = load_resource_extensions(&value)
        }

        Ok(Manifest {
               dir: absolute_dir,
               workspace: workspace,
               runtime: runtime,
           })
    }

    /// Build runtime settings.
    pub fn build<P>(&self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        bincode::serialize(&self.runtime, path)
    }

    /// Get the absolute path to the root folder of workspace.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Get the workspace settings.
    pub fn workspace(&self) -> &WorkspaceSettings {
        &self.workspace
    }

    /// Get the runtime settings.
    pub fn runtime(&self) -> &RuntimeSettings {
        &self.runtime
    }
}

macro_rules! resource_exts_decl {
    ($($name: expr => $resource: ident,)*) => (
        fn load_resource_extensions(value: &toml::Value) -> HashMap<String, ResourceType> {
            let mut types = HashMap::new();
            $(
                if let Some(vec) = load_as_array(&value, $name) {
                    for v in vec.iter().filter_map(|v| v.as_str()) {
                        types.insert(v.trim_matches('.').to_owned(), ResourceType::$resource);
                    }
                }
            )*
            types
        }
    )
}

resource_exts_decl! {
    &["textures"] => Texture,
    &["bytes"] => Bytes,
    &["atlases"] => Atlas,
    &["shaders"] => Shader,
    &["materials"] => Material,
}