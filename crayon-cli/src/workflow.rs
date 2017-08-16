use crayon_workflow;
use std::path::PathBuf;

use errors::*;

pub struct Workflow {
    pub rev: String,
    pub manifest: crayon_workflow::Manifest,
    pub database: crayon_workflow::ResourceDatabase,
}

impl Workflow {
    pub fn new(rev: &str) -> Result<Self> {
        let wd = ::std::env::current_dir()?;
        let manifest = crayon_workflow::Manifest::find(&wd)?;

        Ok(Workflow {
               rev: rev.to_owned(),
               database: crayon_workflow::ResourceDatabase::new(manifest.clone())?,
               manifest: manifest,
           })
    }

    pub fn setup(&self) -> Result<()> {
        ::std::env::set_current_dir(&self.build_path())?;
        Ok(())
    }

    pub fn build_path(&self) -> PathBuf {
        self.manifest.dir().join("build")
    }
}