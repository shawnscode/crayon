use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::any::Any;
use std::io::Read;
use std::fs;

use bincode;
use uuid;

use super::*;

pub enum ResourceItem {
    Bytes(BytesItem),
    Texture(TextureItem),
}

type InstanceId = usize;

/// `ResourceSystem` allows you to find and access resources. When building resources
/// during development, a manifest for all the resources will be generated. You should
/// load the manifest before loading resources.
pub struct ResourceSystem {
    ids: HashMap<uuid::Uuid, InstanceId>,
    paths: HashMap<PathBuf, InstanceId>,
    resources: Vec<manifest::ResourceManifestItem>,

    archives: ArchiveCollection,
    backends: ResourceSystemBackendVec,
}

impl ResourceSystem {
    /// Create a new and empty `ResourceSystem`.
    pub fn new() -> Result<ResourceSystem> {
        /// Add working directory as default search path.
        let mut archives = ArchiveCollection::new();
        archives.register(FilesystemArchive::new(Path::new("./"))?);

        let mut rs = ResourceSystem {
            archives: archives,
            backends: ResourceSystemBackendVec::new(),
            ids: HashMap::new(),
            paths: HashMap::new(),
            resources: Vec::new(),
        };

        /// Register default resources.
        rs.register::<Texture>();
        rs.register::<Bytes>();

        Ok(rs)
    }

    /// Register a new resource type.
    #[inline]
    pub fn register<T>(&mut self)
        where T: Resource + ResourceIndex + 'static
    {
        self.backends.register::<T>();
    }

    /// Set the cache size of resources. `ResourceSystem` will keeps reference to
    /// hot resources if we have spare space.
    pub fn set_cache_size<T>(&mut self, size: usize)
        where T: Resource + ResourceIndex + 'static
    {
        self.backends.index_mut::<T>().register_cache(size);
    }

    /// Load a manifest at path of filesystem directly.
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

    /// Unload unused, there is no external references, resources from memory.
    pub fn unload_unused(&mut self) {
        self.backends.unload_unused()
    }

    /// Load a resource item at path. The path is some kind of readable identifier
    /// instead of actual path in filesystem.
    pub fn load<P>(&mut self, path: P) -> Result<ResourceItem>
        where P: AsRef<Path>
    {
        let (uuid, payload) = {
            if let Some(instance_id) = self.paths.get(path.as_ref()) {
                let item = self.resources.get(*instance_id).unwrap();
                (item.uuid, item.payload)
            } else {
                bail!("Failed to load resource at {:?}, not found in any loaded manifest.",
                      path.as_ref());
            }
        };

        self.load_internal(uuid, payload)
    }

    #[inline]
    pub fn load_texture<P>(&mut self, path: P) -> Result<TextureItem>
        where P: AsRef<Path>
    {
        match self.load(path.as_ref())? {
            ResourceItem::Texture(texture) => Ok(texture),
            _ => bail!("Failed to load texture from {:?}.", path.as_ref()),
        }
    }

    #[inline]
    pub fn load_bytes<P>(&mut self, path: P) -> Result<BytesItem>
        where P: AsRef<Path>
    {
        match self.load(path.as_ref())? {
            ResourceItem::Bytes(bytes) => Ok(bytes),
            _ => bail!("Failed to load bytes from {:?}.", path.as_ref()),
        }
    }

    /// Load a resource with uuid.
    #[inline]
    pub fn load_with_uuid(&mut self, uuid: uuid::Uuid) -> Result<ResourceItem> {
        let payload = {
            if let Some(instance_id) = self.ids.get(&uuid) {
                let item = self.resources.get(*instance_id).unwrap();
                item.payload
            } else {
                bail!("Failed to load resource with {:?}, not found in any loaded manifest.",
                      uuid);
            }
        };

        self.load_internal(uuid, payload)
    }

    #[inline]
    pub fn load_texture_with_uuid(&mut self, uuid: uuid::Uuid) -> Result<TextureItem> {
        match self.load_with_uuid(uuid)? {
            ResourceItem::Texture(texture) => Ok(texture),
            _ => bail!("Failed to load texture with {:?}.", uuid),
        }
    }

    #[inline]
    pub fn load_bytes_with_uuid(&mut self, uuid: uuid::Uuid) -> Result<BytesItem> {
        match self.load_with_uuid(uuid)? {
            ResourceItem::Bytes(bytes) => Ok(bytes),
            _ => bail!("Failed to load bytes with {:?}.", uuid),
        }
    }

    fn load_internal(&mut self,
                     uuid: uuid::Uuid,
                     payload: manifest::ResourcePayload)
                     -> Result<ResourceItem> {
        let uuid = uuid.simple().to_string();
        let path = Path::new(&uuid);

        match payload {
            manifest::ResourcePayload::Texture => {
                self.backends
                    .index_mut::<Texture>()
                    .load::<texture::TextureSerializationPayload, &Path>(&self.archives, path)
                    .map(|v| ResourceItem::Texture(v))
            }
            manifest::ResourcePayload::Bytes => {
                self.backends
                    .index_mut::<Bytes>()
                    .load::<bytes::BytesSerializationPayload, &Path>(&self.archives, path)
                    .map(|v| ResourceItem::Bytes(v))
            }
        }
    }

    /// Load a resource item at path of filesystem directly. This function does not have
    /// any requirements on the manifest, and user have to specify the loader manually.
    #[inline]
    pub fn load_from<L, P>(&mut self, path: P) -> Result<Arc<RwLock<L::Item>>>
        where L: ResourceLoader,
              P: AsRef<Path>
    {
        self.backends
            .index_mut::<L::Item>()
            .load::<L, P>(&self.archives, path)
    }
}

struct ResourceSystemBackendVec(pub Vec<Option<Box<Any>>>);

impl ResourceSystemBackendVec {
    pub fn new() -> Self {
        ResourceSystemBackendVec(Vec::new())
    }

    pub fn index_mut<T>(&mut self) -> &mut ResourceSystemBackend<T>
        where T: Resource + ResourceIndex + 'static
    {
        self.0[T::type_index()]
            .as_mut()
            .expect("Tried to perform an operation on resource type that not registered.")
            .downcast_mut::<ResourceSystemBackend<T>>()
            .unwrap()
    }

    pub fn register<T>(&mut self)
        where T: Resource + ResourceIndex + 'static
    {
        if T::type_index() >= self.0.len() {
            for _ in self.0.len()..(T::type_index() + 1) {
                self.0.push(None)
            }
        }

        // Returns if we are going to register this resource duplicatedly.
        if let Some(_) = self.0[T::type_index()] {
            return;
        }

        self.0[T::type_index()] = Some(Box::new(ResourceSystemBackend::<T>::new()));
    }

    pub fn unload_unused(&mut self) {
        for v in &mut self.0 {
            if let &mut Some(ref mut backend) = v {
                backend
                    .downcast_mut::<Box<ResourceSystemBackendTrait>>()
                    .unwrap()
                    .unload_unused();
            }
        }
    }
}

trait ResourceSystemBackendTrait {
    fn unload_unused(&mut self);
}

impl<T> ResourceSystemBackendTrait for ResourceSystemBackend<T>
    where T: Resource + ResourceIndex + 'static
{
    fn unload_unused(&mut self) {
        self.unload_unused()
    }
}