use clap;

use errors::*;
use cargo;

use std::path::Path;
use std::fs;
use std::io::Write;

const MAIN: &[u8] = include_bytes!("../../template/src/main.rs");
const MANIFEST: &[u8] = include_bytes!("../../template/workspace.toml");

pub fn execute(rev: &str, matches: &clap::ArgMatches) -> Result<()> {
    let path = matches.value_of("path").unwrap();

    // Execute `cargo new -q --bin <path>`.
    cargo::call(&["new", "-q", "--bin", path])?;

    // Append crayon dependency to the project's Cargo.toml.
    {
        let manifest = Path::new(&path).join("Cargo.toml");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(manifest)?;

        file.write(format!("crayon = {{ git = \"https://github.com/shawnscode/crayon\", rev = \"{0}\" }}",
                           rev).as_ref())?;
        file.flush()?;
    }

    // Add default crayon manifest.
    {
        let manifest = Path::new(&path).join("workspace.toml");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(manifest)?;

        file.write(MANIFEST)?;
        file.flush()?;
    }

    // Add default main.rs
    {
        let main = Path::new(&path).join("src").join("main.rs");
        let mut file = fs::OpenOptions::new().create(true).write(true).open(main)?;

        file.write(MAIN)?;
        file.flush()?;
    }

    Ok(())
}