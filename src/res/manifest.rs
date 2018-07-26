use std::io::Read;
use std::path::Path;

use utils::hash_value::HashValue;
use utils::uuid::Uuid;

use super::byteorder::ByteOrderRead;
use super::errors::*;

pub const NAME: &'static str = "MANIFEST";
pub const MAGIC: [u8; 8] = [
    'M' as u8, 'N' as u8, 'F' as u8, 'T' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ManifestItem {
    pub location: HashValue<Path>,
    pub uuid: Uuid,
}

pub struct Manifest {
    pub items: Vec<ManifestItem>,
}

impl Manifest {
    pub fn load(file: &mut dyn Read) -> Result<Manifest> {
        let mut buf = [0; 16];
        file.read_exact(&mut buf[0..8])
            .map_err(|err| Error::Malformed(format!("[ManifestLoader] {:?}", err)))?;

        // MAGIC: [u8; 8]
        if &buf[0..8] != &MAGIC[..] {
            return Err(Error::Malformed(
                "[ManifestLoader] MAGIC number not match.".into(),
            ));
        }

        let len = file.read_u32()?;
        let mut manifest = Manifest {
            items: Vec::with_capacity(len as usize),
        };

        for _ in 0..len {
            manifest.items.push(ManifestItem {
                location: file.read_value(&mut buf)
                    .map_err(|err| Error::Malformed(format!("[ManifestLoader] {:?}", err)))?,
                uuid: file.read_value(&mut buf)
                    .map_err(|err| Error::Malformed(format!("[ManifestLoader] {:?}", err)))?,
            });
        }

        Ok(manifest)
    }
}
