use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;

fn main() {
    let mut scripts = String::new();
    if let Ok(output) = process::Command::new("git")
           .args(&["rev-parse", "--short", "HEAD"])
           .output() {
        if output.status.success() {
            let rev = String::from_utf8(output.stdout).unwrap();
            scripts += &format!("const BUILD_REV : &'static str = \"{0}\";\n", rev.trim());
        }
    }

    /// Write all the environment injections.
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("env.rs");
    let mut file = File::create(&destination).unwrap();
    file.write_all(&scripts.as_ref()).unwrap();
}