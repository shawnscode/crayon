#[macro_use]
extern crate crayon;
extern crate crayon_canvas;
extern crate crayon_imgui;
extern crate image;
extern crate rand;

use std::env;
use std::io;
use std::io::prelude::*;
use std::process::exit;

mod utils;
mod texture;
mod render_target;
mod canvas;
mod imgui;

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

    let name = &args[1];
    match &name[..] {
        "texture" => texture::main(&args[1..]),
        "render_target" => render_target::main(&args[1..]),
        "canvas" => canvas::main(&args[1..]),
        "imgui" => imgui::main(&args[1..]),
        _ => usage(),
    }
}