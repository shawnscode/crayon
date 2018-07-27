use std::io::Write;

use crayon::res::byteorder::ByteOrderWrite;
use crayon::res::manifest::{self, Manifest};

use errors::*;

pub fn compile(man: &Manifest, o: &mut dyn Write) -> Result<()> {
    let io_err = |err| Error::Compile(format!("[ManifestCompiler] {}", err));

    // MAGIC: [u8; 8]
    o.write_all(&manifest::MAGIC).map_err(io_err)?;

    // LEN
    o.write_u32(man.items.len() as u32).map_err(io_err)?;

    // ITEMS
    for v in &man.items {
        o.write_value(v.location).map_err(io_err)?;
        o.write_value(v.uuid).map_err(io_err)?;
    }

    Ok(())
}
