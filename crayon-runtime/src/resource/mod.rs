pub mod errors;
pub mod archive;
pub mod cache;
pub mod backend;
pub mod manifest;

pub mod texture;
pub mod bytes;

pub use self::archive::{File, Archive, FilesystemArchive, ZipArchive, ArchiveCollection};
pub use self::cache::Cache;
pub use self::backend::{ResourceSystemBackend, Resource, ResourceLoader};
pub use self::errors::*;

pub use self::texture::Texture;
pub use self::bytes::Bytes;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::fs;
use std::io::Read;
use std::collections::HashMap;

use bincode;
use uuid;

pub type TextureItem = Arc<RwLock<Texture>>;
pub type BytesItem = Arc<RwLock<Bytes>>;

pub enum ResourceItem {
    Bytes(BytesItem),
    Texture(TextureItem),
}

type InstanceId = usize;

pub struct ResourceSystem {
    ids: HashMap<uuid::Uuid, InstanceId>,
    paths: HashMap<PathBuf, InstanceId>,
    resources: Vec<manifest::ResourceManifestItem>,

    archives: ArchiveCollection,
    textures: ResourceSystemBackend<Texture>,
    bytes: ResourceSystemBackend<Bytes>,
}

impl ResourceSystem {
    pub fn new() -> ResourceSystem {
        ResourceSystem {
            textures: ResourceSystemBackend::new(),
            bytes: ResourceSystemBackend::new(),

            archives: ArchiveCollection::new(),
            ids: HashMap::new(),
            paths: HashMap::new(),
            resources: Vec::new(),
        }
    }

    pub fn load_manifest<P>(&mut self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        if !path.as_ref().exists() {
            bail!("Failed to load manifest at path {:?}.", path.as_ref());
        }

        let mut file = fs::OpenOptions::new().read(true).open(path.as_ref())?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let mut manifest = bincode::deserialize::<manifest::ResourceManifest>(&bytes)?;

        /// Append path to archive collection.
        let archive = FilesystemArchive::new(path.as_ref().parent().unwrap().join(&manifest.path))?;
        self.archives.register::<FilesystemArchive>(archive);

        /// Append resources listed in manifest.
        for (uid, item) in manifest.items.drain() {
            if !self.ids.contains_key(&uid) {
                self.ids.insert(uid, self.resources.len());
                self.paths.insert(item.path.clone(), self.resources.len());
                self.resources.push(item);
            }
        }

        Ok(())
    }

    pub fn unload_unused(&mut self) {
        self.textures.unload_unused();
    }

    pub fn load<P>(&mut self, path: P) -> Result<ResourceItem>
        where P: AsRef<Path>
    {
        let item = {
            if let Some(instance_id) = self.paths.get(path.as_ref()) {
                self.resources.get(*instance_id).unwrap()
            } else {
                bail!("Failed to load texture at {:?}, not found in any loaded manifest.",
                      path.as_ref());
            }
        };

        let uuid = item.uuid.simple().to_string();
        let path = Path::new(&uuid);

        match item.payload {
            manifest::ResourcePayload::Texture => {
                self.textures
                    .load::<Texture, &Path>(&self.archives, path)
                    .map(|v| ResourceItem::Texture(v))
            }
            manifest::ResourcePayload::Bytes => {
                self.bytes
                    .load::<Bytes, &Path>(&self.archives, path)
                    .map(|v| ResourceItem::Bytes(v))
            }
        }
    }

    pub fn load_texture<P>(&mut self, path: P) -> Result<TextureItem>
        where P: AsRef<Path>
    {
        match self.load(path.as_ref())? {
            ResourceItem::Texture(texture) => Ok(texture),
            _ => bail!("Failed to load texture from {:?}.", path.as_ref()),
        }
    }

    pub fn load_bytes<P>(&mut self, path: P) -> Result<BytesItem>
        where P: AsRef<Path>
    {
        match self.load(path.as_ref())? {
            ResourceItem::Bytes(bytes) => Ok(bytes),
            _ => bail!("Failed to load bytes from {:?}.", path.as_ref()),
        }
    }
}