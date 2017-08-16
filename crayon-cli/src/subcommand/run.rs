use clap;

use errors::*;
use cargo;

use workflow::Workflow;
use crayon_workflow::platform;

pub fn execute(mut workflow: &mut Workflow, matches: &clap::ArgMatches) -> Result<()> {
    let path = workflow.build_path().join("resources");

    workflow.database.refresh()?;
    workflow
        .database
        .build(&workflow.rev, platform::BuildTarget::MacOS, &path)?;

    let mut args = vec!["run", "--color=always"];

    if matches.is_present("release") {
        args.push("--release");
    }

    cargo::call(&args)
}