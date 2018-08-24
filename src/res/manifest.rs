use std::io::Read;
use std::path::Path;

use bincode;
use uuid;

use errors::*;
use utils::hash_value::HashValue;

pub const NAME: &'static str = ".MANIFEST";
pub const MAGIC: [u8; 8] = [
    'M' as u8, 'N' as u8, 'F' as u8, 'T' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ManifestItem {
    pub location: HashValue<Path>,
    pub uuid: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Manifest {
    pub items: Vec<ManifestItem>,
}

impl Manifest {
    pub fn new() -> Self {
        Manifest { items: Vec::new() }
    }

    pub fn load(mut file: &mut dyn Read) -> Result<Manifest> {
        let mut buf = [0; 16];
        file.read_exact(&mut buf[0..8])?;

        // MAGIC: [u8; 8]
        if &buf[0..8] != &MAGIC[..] {
            bail!("[ManifestLoader] MAGIC number not match.");
        }

        let manifest = bincode::deserialize_from(&mut file)?;
        Ok(manifest)
    }
}
