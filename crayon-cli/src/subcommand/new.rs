use clap;

use errors::*;
use cargo;

use std::path::Path;
use std::fs;
use std::io::Write;

const MANIFEST: &[u8] = include_bytes!("../../template/Crayon.toml");

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

        file.write(format!("crayon = {{ git = \"https://github.com/kaisc/crayon\", rev = \"{0}\" }}",
                           rev).as_ref())?;
        file.flush()?;
    }

    // Add default crayon manifest.
    {
        let manifest = Path::new(&path).join("Crayon.toml");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(manifest)?;

        file.write(MANIFEST)?;
        file.flush()?;
    }

    Ok(())
}