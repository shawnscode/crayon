use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;

fn main() {
    let mut scripts = String::new();

    /// Write all the environment injections.
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("env.rs");
    let mut file = File::create(&destination).unwrap();
    file.write_all(&scripts.as_ref()).unwrap();
}