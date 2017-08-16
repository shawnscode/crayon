use clap;

use errors::*;
use cargo;

use workflow::Workflow;
use crayon_workflow::platform;

pub fn execute(mut workflow: &mut Workflow, matches: &clap::ArgMatches) -> Result<()> {
    let build_path = workflow.build_path();

    workflow.database.refresh()?;
    workflow
        .database
        .build(&workflow.rev,
               platform::BuildTarget::MacOS,
               &build_path.join("resources"))?;

    let mut args = vec!["run", "--color=always"];

    if matches.is_present("release") {
        args.push("--release");
    }

    cargo::call(&args)
}