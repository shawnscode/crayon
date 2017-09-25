use crayon_workflow::prelude::*;

use errors::*;

pub struct Workflow {
    pub rev: String,
    pub workspace: Workspace,
}

impl Workflow {
    pub fn new(rev: &str) -> Result<Self> {
        let wd = ::std::env::current_dir()?;
        Ok(Workflow {
               rev: rev.to_owned(),
               workspace: Workspace::find(&wd)?,
           })
    }

    pub fn setup(&self) -> Result<()> {
        let path = self.workspace.manifest.dir().join("build");
        if !path.exists() {
            ::std::fs::create_dir_all(&path)?;
        }

        Ok(())
    }

    pub fn build(&self) -> Result<()> {
        let path = self.workspace
            .manifest
            .dir()
            .join("build")
            .join("resources");

        self.workspace.build(BuildTarget::MacOS, path)?;
        Ok(())
    }
}