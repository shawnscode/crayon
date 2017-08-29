use crayon_workflow;

use errors::*;

pub struct Workflow {
    pub rev: String,
    pub workspace: crayon_workflow::Workspace,
}

impl Workflow {
    pub fn new(rev: &str) -> Result<Self> {
        let wd = ::std::env::current_dir()?;
        Ok(Workflow {
               rev: rev.to_owned(),
               workspace: crayon_workflow::Workspace::find(&wd)?,
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

        self.workspace
            .build(crayon_workflow::BuildTarget::MacOS, path)?;
        Ok(())
    }
}