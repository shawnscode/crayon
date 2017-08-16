use clap;

use errors::*;
use cargo;

use workflow::Workflow;
use crayon_workflow::platform;

pub fn execute(mut workflow: &mut Workflow, matches: &clap::ArgMatches) -> Result<()> {
    workflow.database.refresh()?;
    workflow
        .database
        .build(&workflow.rev,
               platform::BuildTarget::MacOS,
               "build/resources")?;

    let mut args = vec!["build", "--color=always"];

    if matches.is_present("release") {
        args.push("--release");
    }

    cargo::call(&args)
}