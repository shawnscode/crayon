use clap;

use errors::*;
use cargo;

pub fn execute(matches: &clap::ArgMatches) -> Result<()> {
    let mut args = vec!["build", "--color=always"];

    if matches.is_present("release") {
        args.push("--release");
    }

    cargo::call(&args)
}