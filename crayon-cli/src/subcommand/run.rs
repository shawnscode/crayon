use clap;
use crayon_workflow;

use errors::*;
use cargo;

use resource;

pub fn execute(man: &crayon_workflow::Manifest, matches: &clap::ArgMatches) -> Result<()> {
    resource::refresh(&man)?;

    let mut args = vec!["run", "--color=always"];

    if matches.is_present("release") {
        args.push("--release");
    }

    cargo::call(&args)
}