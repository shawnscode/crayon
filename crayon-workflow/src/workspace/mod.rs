//! # Workspace

pub mod manifest;
pub mod database;

pub use self::manifest::{Manifest, WorkspaceSettings};
pub use self::database::Database;

use std::fs;
use std::path::Path;

use errors::*;
use prelude::*;

pub struct Workspace {
    pub manifest: Manifest,
    pub database: Database,
}

impl Workspace {
    pub fn find<P>(path: P) -> Result<Workspace>
        where P: AsRef<Path>
    {
        if let Ok(dir) = fs::metadata(&path) {
            if dir.is_dir() {
                let file_path = path.as_ref().join("workspace.toml");
                if let Ok(file) = fs::metadata(&file_path) {
                    if file.is_file() {
                        return Workspace::load_from(&file_path);
                    }
                } else {
                    if let Some(parent) = path.as_ref().parent() {
                        return Workspace::find(parent);
                    }
                }
            }
        }

        bail!(ErrorKind::WorkspaceNotFound);
    }

    pub fn load_from<P>(path: P) -> Result<Workspace>
        where P: AsRef<Path>
    {
        let manifeset = Manifest::load_from(path)?;
        let projs = manifeset.dir().join(".crayon");
        fs::create_dir_all(&projs)?;

        let mut database = Database::load_from(&projs)?;
        database.refresh(&manifeset.workspace())?;

        Ok(Workspace {
               manifest: manifeset,
               database: database,
           })
    }

    pub fn build<P>(&self, os: BuildTarget, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        fs::create_dir_all(path.as_ref())?;

        self.database
            .build("0.0.5", os, path.as_ref(), &self.manifest.workspace())?;
        self.manifest.build(path.as_ref().join("configs"))?;

        Ok(())
    }

    pub fn load_with_desc<P>(&self, path: P, desc: ResourceMetadataDesc) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        self.database.load_with_desc(path, desc)
    }

    pub fn save(&self) -> Result<()> {
        self.database.save(self.manifest.dir().join(".crayon"))?;
        Ok(())
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        self.save().unwrap();
    }
}