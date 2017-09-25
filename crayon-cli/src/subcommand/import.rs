use clap;
use crayon_workflow::prelude::*;

use errors::*;
use workflow::Workflow;

pub fn execute(workflow: &mut Workflow, matches: &clap::ArgMatches) -> Result<()> {
    let path = matches.value_of("path").unwrap();

    let _metadata = if let Some(tt) = matches.value_of("type") {
        let tt = tt.to_lowercase();
        if tt == "atlas" {
            workflow
                .workspace
                .database
                .reimport(path, ResourceType::Atlas)?
        } else {
            bail!("not supports!");
        }
    } else {
        workflow
            .workspace
            .database
            .import(path, &workflow.workspace.manifest.workspace())?
    };

    Ok(())
}