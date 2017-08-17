use clap;

use errors::*;
use cargo;

use workflow::Workflow;
use crayon_workflow::platform;

pub fn compile(mut workflow: &mut Workflow) -> Result<()> {
    {
        let path = workflow.build_path().join("resources");
        workflow.database.refresh()?;
        workflow
            .database
            .build(&workflow.rev, platform::BuildTarget::MacOS, &path)?;
    }

    {
        let path = workflow.build_path().join("configs");
        workflow.manifest.save_settings(path)?;
    }

    Ok(())
}

pub fn execute(mut workflow: &mut Workflow, matches: &clap::ArgMatches) -> Result<()> {
    compile(&mut workflow)?;

    let mut args = vec!["build", "--color=always"];

    if matches.is_present("release") {
        args.push("--release");
    }

    cargo::call(&args)
}