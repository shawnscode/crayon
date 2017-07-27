pub mod errors;
pub mod archive;
pub mod cache;
pub mod resource;
pub mod texture;

pub use self::archive::{Read, Seek, File, Archive, FilesystemArchive, ZipArchive,
                        ArchiveCollection};
pub use self::cache::Cache;
pub use self::texture::{Texture, TextureMetadata};
pub use self::resource::{ResourceSystem, Resource, ResourceLoader};

use std::sync::{Arc, RwLock};

pub type TextureItem = Arc<RwLock<Texture>>;

pub struct ResourceSystemTable {
    pub textures: ResourceSystem<Texture>,
}

impl ResourceSystemTable {
    pub fn new() -> ResourceSystemTable {
        ResourceSystemTable { textures: ResourceSystem::new() }
    }
}