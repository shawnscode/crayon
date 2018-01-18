#[macro_use]
extern crate crayon;
extern crate crayon_imgui;
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

const USAGE: &'static str = "";

fn usage() -> ! {
    let _ = writeln!(&mut io::stderr(), "{}", USAGE);
    exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
    }

    let name = args[1].clone();
    match args[1].as_str() {
        "texture" => texture::main(name, &args[1..]),
        "render_target" => render_target::main(name, &args[1..]),
        "imgui" => imgui::main(name, &args[1..]),
        "input" => input::main(name, &args[1..]),
        _ => usage(),
    }
}
