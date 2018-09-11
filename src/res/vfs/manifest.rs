//! Manifest for all the AssetBundles in the build.

use std::io::Read;
use std::path::PathBuf;

use bincode;
use uuid::Uuid;

use errors::*;
use utils::{DataBuffer, DataBufferPtr, FastHashMap, HashValue};

pub const NAME: &'static str = ".MANIFEST";
pub const MAGIC: [u8; 8] = [
    'M' as u8, 'N' as u8, 'F' as u8, 'T' as u8, ' ' as u8, 0, 0, 1,
];

/// A manifest item in the build.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct ManifestItem {
    pub filename: DataBufferPtr<str>,
    pub dependencies: DataBufferPtr<[usize]>,
    pub uuid: Uuid,
}

/// Manifest for all the resources in the build.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Manifest {
    pub items: Vec<ManifestItem>,
    pub buf: DataBuffer,

    #[serde(skip)]
    uuids: FastHashMap<Uuid, usize>,
    #[serde(skip)]
    filenames: FastHashMap<HashValue<str>, usize>,
}

impl Manifest {
    pub fn new() -> Self {
        Manifest {
            items: Vec::new(),
            buf: DataBuffer::with_capacity(0),
            uuids: FastHashMap::default(),
            filenames: FastHashMap::default(),
        }
    }

    pub fn load_from(mut file: &mut dyn Read) -> Result<Manifest> {
        let mut buf = [0; 16];
        file.read_exact(&mut buf[0..8])?;

        // MAGIC: [u8; 8]
        if &buf[0..8] != &MAGIC[..] {
            bail!("[ManifestLoader] MAGIC number not match.");
        }

        let mut manifest: Manifest = bincode::deserialize_from(&mut file)?;

        manifest.uuids.clear();
        manifest.filenames.clear();

        for (index, v) in manifest.items.iter_mut().enumerate() {
            manifest.uuids.insert(v.uuid, index);

            let filename = manifest.buf.as_str(v.filename);
            manifest.filenames.insert(filename.into(), index);
        }

        Ok(manifest)
    }
}

impl Manifest {
    #[inline]
    pub fn redirect<T>(&self, filename: T) -> Option<Uuid>
    where
        T: AsRef<str>,
    {
        self.filenames
            .get(&filename.as_ref().into())
            .map(|&index| self.items[index].uuid)
    }

    #[inline]
    pub fn locate(&self, uuid: Uuid) -> Option<PathBuf> {
        if self.uuids.contains_key(&uuid) {
            Some(format!("{:X}", uuid.simple()).into())
        } else {
            None
        }
    }

    #[inline]
    pub fn contains(&self, uuid: Uuid) -> bool {
        self.uuids.contains_key(&uuid)
    }

    #[inline]
    pub fn dependencies(&self, uuid: Uuid) -> Option<Dependencies> {
        self.uuids.get(&uuid).map(|&index| Dependencies {
            index: 0,
            dependencies: self.buf.as_slice(self.items[index].dependencies),
            items: self.items.as_ref(),
        })
    }
}

pub struct Dependencies<'a> {
    index: usize,
    dependencies: &'a [usize],
    items: &'a [ManifestItem],
}

impl<'a> Iterator for Dependencies<'a> {
    type Item = Uuid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.dependencies.len() {
            self.index += 1;
            Some(self.items[self.index - 1].uuid)
        } else {
            None
        }
    }
}
