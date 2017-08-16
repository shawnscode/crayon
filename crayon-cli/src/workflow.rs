use crayon_workflow;

use errors::*;

pub struct Workflow {
    pub manifest: crayon_workflow::Manifest,
    pub database: crayon_workflow::ResourceDatabase,
}

impl Workflow {
    pub fn new() -> Result<Self> {
        let wd = ::std::env::current_dir()?;
        let manifest = crayon_workflow::Manifest::find(&wd)?;

        Ok(Workflow {
               database: crayon_workflow::ResourceDatabase::new(manifest.clone())?,
               manifest: manifest,
           })
    }
}