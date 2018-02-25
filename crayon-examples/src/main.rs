#[macro_use]
extern crate crayon;
extern crate crayon_imgui;
extern crate crayon_scene;

#[macro_use]
extern crate failure;
extern crate image;
extern crate obj;
extern crate rand;

use std::env;
use std::io;
use std::io::prelude::*;
use std::process::exit;

mod utils;
mod texture;
mod render_target;
mod imgui;
mod input;
mod mesh;
mod errors;

const USAGE: &str = "";

fn usage() -> ! {
    let _ = writeln!(&mut io::stderr(), "{}", USAGE);
    exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
    }

    let mut setup = crayon::application::Settings::default();
    setup.window.title = args[1].clone();

    if args.len() > 2 && "headless" == args[2] {
        setup.headless = true;
    }

    match args[1].as_str() {
        "texture" => texture::main(setup),
        "render_target" => render_target::main(setup),
        "imgui" => imgui::main(setup),
        "input" => input::main(setup),
        "mesh" => mesh::main(setup),
        _ => usage(),
    }
}
