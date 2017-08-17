#[macro_use]
extern crate error_chain;
extern crate clap;

extern crate crayon;
extern crate crayon_workflow;

mod errors;
mod cargo;
mod subcommand;
mod workflow;

include!(concat!(env!("OUT_DIR"), "/env.rs"));

fn main() {
    let cmd_new = clap::SubCommand::with_name("new")
        .about("Create a new crayon project")
        .arg(clap::Arg::with_name("path")
                 .short("p")
                 .required(true)
                 .index(1)
                 .help("Set the project path"));

    let cmd_build = clap::SubCommand::with_name("build")
        .about("Compile the current project and its resources")
        .arg(clap::Arg::with_name("release")
                 .short("r")
                 .help("Build artifacts in release mode, with optimizations"));

    let cmd_run = clap::SubCommand::with_name("run")
        .about("Build and execute src/main.rs")
        .arg(clap::Arg::with_name("release")
                 .short("r")
                 .help("Build artifacts in release mode, with optimizations"))
        .arg(clap::Arg::with_name("manifest")
                 .short("m")
                 .help("Choose crayon manifest to build"));

    let matches = clap::App::new("crayon-cli")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(cmd_new)
        .subcommand(cmd_build)
        .subcommand(cmd_run)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        subcommand::new::execute(&BUILD_REV, matches).unwrap();
        return;
    }

    let mut workflow = workflow::Workflow::new(&BUILD_REV).unwrap();
    workflow.setup().unwrap();

    if let Some(matches) = matches.subcommand_matches("build") {
        subcommand::build::execute(&mut workflow, matches).unwrap();
        return;
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        subcommand::run::execute(&mut workflow, matches).unwrap();
        return;
    }
}