#[macro_use]
extern crate clap;

use std::process;
use std::path;

fn main() {
    let matches = clap_app!(crayonctl => 
        (version: "0.0.1")
        (author: "Jingkai Mao <oammix@gmail.com>")
        (@subcommand new =>
            (about: "Create a new crayon project")
            (@arg path: +required)
        ))
            .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        create(matches.value_of("path").unwrap());
    }
}

fn create(path: &str) {
    process::Command::new("cargo")
        .args(&["new",
                "--template",
                "https://github.com/kaisc/crayon/tree/workflow/crayon-workflow/template",
                path])
        .output()
        .expect("failed create new project with cargo.");
}