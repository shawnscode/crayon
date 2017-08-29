//! # Workspace
//!
//! We use workspace to define where in your computer's file system to store your project.
//!
//! Crayon automatically imports resources and manage various kinds of additional data
//! about them for you. The whole works is configuable by editing a manifest file, feel
//! free to checkout the `manifest` module for details.
pub mod manifest;
pub mod database;

pub use self::manifest::{Manifest, WorkspaceSettings};
pub use self::database::Database;

use std::fs;
use std::path::Path;

use super::*;

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

        let mut database = Database::load_from(manifeset.dir().join(".crayon"))?;
        database.refresh(&manifeset.workspace())?;

        Ok(Workspace {
               manifest: manifeset,
               database: database,
           })
    }

    pub fn build<P>(&self, os: platform::BuildTarget, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        fs::create_dir_all(path.as_ref())?;

        self.database
            .build("0.0.3", os, path.as_ref(), &self.manifest.workspace())?;
        self.manifest.build(path.as_ref().join("configs"))?;

        Ok(())
    }

    pub fn reimport<P>(&self, path: P, tt: Resource) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        self.database.reimport(path, tt)
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