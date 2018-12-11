//! Manifest for all the AssetBundles in the build.

use std::io::Read;

use bincode;
use inlinable_string::{InlinableString, StringExt};
use uuid::Uuid;

use crate::errors::*;
use crate::utils::prelude::{DataBuffer, DataBufferPtr, FastHashMap, HashValue};

pub const NAME: &str = "MANIFEST";
pub const MAGIC: [u8; 8] = [b'M', b'N', b'F', b'T', b' ', 0, 0, 1];

/// A manifest item in the build.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct ManifestItem {
    pub filename: DataBufferPtr<str>,
    pub dependencies: DataBufferPtr<[usize]>,
    pub uuid: Uuid,
}

/// Manifest for all the resources in the build.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Manifest {
    pub items: Vec<ManifestItem>,
    pub buf: DataBuffer,
}

impl Manifest {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_from(mut file: &mut dyn Read) -> Result<Manifest> {
        let mut buf = [0; 16];
        file.read_exact(&mut buf[0..8])?;

        // MAGIC: [u8; 8]
        if buf[0..8] != MAGIC[..] {
            bail!("[ManifestLoader] MAGIC number not match.");
        }

        Ok(bincode::deserialize_from(&mut file)?)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ManfiestResolver {
    manifests: Vec<Manifest>,
    manifest_prefixs: Vec<InlinableString>,
    uuids: FastHashMap<Uuid, (usize, usize)>,
    filenames: FastHashMap<HashValue<str>, Uuid>,
}

impl ManfiestResolver {
    pub fn new() -> Self {
        ManfiestResolver {
            manifests: Vec::new(),
            manifest_prefixs: Vec::new(),
            uuids: FastHashMap::default(),
            filenames: FastHashMap::default(),
        }
    }

    pub fn add<T: Into<InlinableString>>(&mut self, prefix: T, file: &mut dyn Read) -> Result<()> {
        let manifest = Manifest::load_from(file)?;

        let mut prefix = prefix.into();
        if !prefix.ends_with('/') {
            prefix.push('/');
        }

        let index = self.manifests.len();
        for (sub_index, v) in manifest.items.iter().enumerate() {
            let filename = manifest.buf.as_str(v.filename);
            let fullname = format!("{}{}", prefix, filename);

            self.uuids.insert(v.uuid, (index, sub_index));
            self.filenames.insert(fullname.into(), v.uuid);
        }

        self.manifests.push(manifest);
        self.manifest_prefixs.push(prefix);
        Ok(())
    }

    /// Checks if the uuid exists in this registry.
    #[inline]
    pub fn contains(&self, uuid: Uuid) -> bool {
        self.uuids.contains_key(&uuid)
    }

    /// Return the UUID if the fullname exists in this registry.
    #[inline]
    pub fn find<T: AsRef<str>>(&self, fullname: T) -> Option<Uuid> {
        let fullname = fullname.as_ref().into();
        self.filenames.get(&fullname).cloned()
    }

    /// Resolve the UUID to full path of corresponding resource.
    #[inline]
    pub fn resolve(&self, uuid: Uuid) -> Option<String> {
        self.uuids
            .get(&uuid)
            .and_then(|&(index, _)| self.manifest_prefixs.get(index))
            .map(|prefix| format!("{}/{:X}", prefix, uuid.to_simple()))
    }

    /// Return the iterator over all the dependencies of specified resource if exists.
    #[inline]
    pub fn dependencies(&self, uuid: Uuid) -> Option<Dependencies> {
        self.uuids.get(&uuid).and_then(|&(index, sub_index)| {
            self.manifests.get(index).map(|manifest| {
                let dependencies = manifest.items[sub_index].dependencies;
                Dependencies {
                    index: 0,
                    dependencies: manifest.buf.as_slice(dependencies),
                    items: manifest.items.as_ref(),
                }
            })
        })
    }
}

/// An iterator visiting all the dependencies of specified resource.
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
