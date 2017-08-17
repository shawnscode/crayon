use clap;

use errors::*;

use crayon_workflow;
use workflow::Workflow;

pub fn execute(workflow: &mut Workflow, matches: &clap::ArgMatches) -> Result<()> {
    let path = matches.value_of("path").unwrap();

    let _metadata = if let Some(tt) = matches.value_of("type") {
        let tt = tt.to_lowercase();
        if tt == "atlas" {
            workflow
                .database
                .load_metadata_as(path, crayon_workflow::Resource::Atlas)?
        } else {
            bail!("not supports!");
        }
    } else {
        workflow.database.load_metadata(path)?
    };

    Ok(())
}