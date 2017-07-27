pub mod errors;
pub mod archive;
pub mod cache;
pub mod resource;
pub mod texture;
pub mod serialization;

pub use self::archive::{File, Archive, FilesystemArchive, ZipArchive, ArchiveCollection};
pub use self::cache::Cache;
pub use self::texture::Texture;
pub use self::resource::{ResourceSystem, Resource, ResourceLoader};
pub use self::errors::*;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::fs;
use std::io::Read;
use std::collections::HashMap;

use bincode;
use uuid;

pub type TextureItem = Arc<RwLock<Texture>>;

type InstanceId = usize;

pub struct ResourceSystemTable {
    ids: HashMap<uuid::Uuid, InstanceId>,
    paths: HashMap<PathBuf, InstanceId>,
    resources: Vec<serialization::ResourceManifestItem>,

    archives: ArchiveCollection,
    textures: ResourceSystem<Texture>,
}

impl ResourceSystemTable {
    pub fn new() -> ResourceSystemTable {
        ResourceSystemTable {
            textures: ResourceSystem::new(),
            archives: ArchiveCollection::new(),
            ids: HashMap::new(),
            paths: HashMap::new(),
            resources: Vec::new(),
        }
    }

    pub fn load_manifest<P>(&mut self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        let mut file = fs::OpenOptions::new().read(true).open(path.as_ref())?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let mut manifest = bincode::deserialize::<serialization::ResourceManifest>(&bytes)?;

        /// Append path to archive collection.
        let archive = FilesystemArchive::new(path.as_ref().join(&manifest.path))?;
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

    pub fn load_texture<P>(&mut self, path: P) -> Result<TextureItem>
        where P: AsRef<Path>
    {
        let resources = &self.resources;
        let item = {
            if let Some(instance_id) = self.paths.get(path.as_ref()) {
                resources.get(*instance_id).unwrap()
            } else {
                bail!("Failed to load texture at {:?}, not found in any loaded manifest.",
                      path.as_ref());
            }
        };

        self.textures
            .load::<Texture, &PathBuf>(&self.archives, &item.path)
    }

    // pub fn set_texture_cache_size()
    // pub fn load_texture_with_uuid(&mut self, uuid: uuid::Uuid) -> Result<TextureItem> {
    //     let resources = &self.resources;
    //     let item = {
    //         if let Some(instance_id) = self.ids.get(&uuid) {
    //             resources.get(*instance_id).unwrap()
    //         } else {
    //             bail!("Failed to load texture with {:?}, not found in any loaded manifest.",
    //                   uuid);
    //         }
    //     };

    //     self.textures
    //         .load::<Texture, &PathBuf>(&self.archives, &item.path)
    // }
}