use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;

use toml;

use errors::*;
use resource::Resource;

/// Workflow manifest of crayon project.
#[derive(Debug)]
pub struct Manifest {
    pub dir: PathBuf,
    pub resources: Vec<PathBuf>,
    pub types: HashMap<String, Resource>,
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

    fn parse(path: &Path) -> Result<Manifest> {
        if let Ok(mut file) = fs::File::open(path) {
            let mut raw = String::new();
            file.read_to_string(&mut raw)?;

            let value: toml::Value = raw.parse()?;

            let mut manifest = Manifest {
                dir: path.parent().unwrap().to_owned(),
                resources: Vec::new(),
                types: HashMap::new(),
            };

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
                                    .insert(v.trim_matches('.').to_owned(), Resource::Binary);
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
                }
            }

            Ok(manifest)
        } else {
            bail!("Crayon.toml at {:?} is not valid.", path);
        }
    }
}