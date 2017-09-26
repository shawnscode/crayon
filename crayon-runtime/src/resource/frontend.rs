use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::any::Any;
use std::io::Read;
use std::fs;

use bincode;
use uuid::Uuid;

use utils::hash::HashValue;
use super::*;
use super::errors::*;

/// `ResourceFrontend` allows you to find and access resources. When building resources
/// during development, a manifest for all the resources will be generated. You should
/// load the manifest before loading resources.
pub struct ResourceFrontend {
    paths: HashMap<HashValue<Path>, Uuid>,
    resources: HashMap<Uuid, workflow::ResourceManifestItem>,

    locks: HashSet<Uuid>,

    archives: ArchiveCollection,
    backends: ResourceBackends,
}

impl ResourceFrontend {
    /// Create a new and empty `ResourceFrontend`.
    pub fn new() -> Result<ResourceFrontend> {
        /// Add working directory as default search path.
        let mut archives = ArchiveCollection::new();
        archives.register(FilesystemArchive::new(Path::new("./"))?);

        let mut rs = ResourceFrontend {
            paths: HashMap::new(),
            resources: HashMap::new(),
            locks: HashSet::new(),

            archives: archives,
            backends: ResourceBackends::new(),
        };

        /// Register default resources.
        workflow::register(&mut rs);
        Ok(rs)
    }

    /// Register a new resource type.
    #[inline]
    pub fn register<T>(&mut self)
        where T: Resource + ResourceIndex + 'static
    {
        self.backends.register::<T>();
    }

    /// Set the cache size of resources. `ResourceFrontend` will keeps reference to
    /// hot resources if we have spare space.
    pub fn set_cache_size<T>(&mut self, size: usize)
        where T: Resource + ResourceIndex + 'static
    {
        self.backends.index_mut::<T>().register_cache(size);
    }

    /// Unload unused, there is no external references, resources from memory.
    pub fn unload_unused(&mut self) {
        self.backends.unload_unused();
    }

    /// Load a manifest at path of filesystem directly.
    pub fn load_manifest<P>(&mut self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        if !path.as_ref().exists() {
            bail!(ErrorKind::NotFound);
        }

        let mut file = fs::OpenOptions::new().read(true).open(path.as_ref())?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let mut manifest = bincode::deserialize::<workflow::ResourceManifest>(&bytes)?;

        /// Append path to archive collection.
        let archive = FilesystemArchive::new(path.as_ref().parent().unwrap().join(&manifest.path))?;
        self.archives.register::<FilesystemArchive>(archive);

        /// Append resources listed in manifest.
        for (uuid, item) in manifest.items.drain() {
            if !self.resources.contains_key(&uuid) {
                self.paths.insert((&item.path).into(), uuid);
                self.resources.insert(uuid, item);
            }
        }

        Ok(())
    }

    /// Load a build in resource item at virtual path which should be redirected with
    /// manifest. The path is some kind of readable identifier instead of actual path
    /// in filesystem.
    pub fn load<T, P>(&mut self, path: P) -> Result<ResourcePtr<T>>
        where T: workflow::BuildinResource + 'static,
              P: AsRef<Path>
    {
        let hash = path.as_ref().into();
        let uuid = *self.paths.get(&hash).ok_or(ErrorKind::NotFound)?;
        self.load_with_uuid(uuid)
    }

    /// Load a resource with uuid.
    #[inline]
    pub fn load_with_uuid<T>(&mut self, uuid: Uuid) -> Result<ResourcePtr<T>>
        where T: workflow::BuildinResource + 'static
    {
        let payload = self.resources
            .get(&uuid)
            .and_then(|v| Some(v.payload))
            .ok_or(ErrorKind::NotFound)?;

        if payload != T::payload() {
            bail!(ErrorKind::ResourceDeclarationMismath);
        }

        let uuid_str = uuid.simple().to_string();
        let path = Path::new(&uuid_str);
        if let Some(rc) = self.backends.index_mut::<T>().get(&path) {
            return Ok(rc);
        }

        // Check circular references.
        if self.locks.contains(&uuid) {
            bail!(ErrorKind::CircularReferenceFound);
        }

        let rc = {
            self.locks.insert(uuid);
            let mut file = self.archives.open(&path)?;
            let resource = T::Loader::load_from_file(self, file.as_mut())?;
            self.locks.remove(&uuid);

            Arc::new(RwLock::new(resource))
        };

        self.backends
            .index_mut::<T>()
            .insert(&path, rc.clone())
            .unwrap();
        Ok(rc)
    }

    #[inline]
    pub fn insert<T, P>(&mut self, path: P, item: T) -> Result<ResourcePtr<T>>
        where T: Resource + ResourceIndex + 'static,
              P: AsRef<Path>
    {
        let resource = Arc::new(RwLock::new(item));
        self.backends
            .index_mut::<T>()
            .insert(&path, resource.clone())?;
        Ok(resource)
    }

    #[inline]
    pub fn get<T, P>(&mut self, path: P) -> Option<ResourcePtr<T>>
        where T: Resource + ResourceIndex + 'static,
              P: AsRef<Path>
    {
        self.backends.index_mut::<T>().get(&path)
    }

    /// Load resource at path with custom loader.
    #[inline]
    pub fn load_custom<T, P>(&mut self, path: P) -> Result<ResourcePtr<T::Item>>
        where T: ResourceLoader,
              P: AsRef<Path>
    {
        if let Some(rc) = self.backends.index_mut::<T::Item>().get(&path) {
            return Ok(rc);
        }

        let hash = path.as_ref().into();

        let uuid = if let Some(&uuid) = self.paths.get(&hash) {
            uuid
        } else {
            let uuid = Uuid::new_v4();
            self.paths.insert(hash, uuid);
            uuid
        };

        if self.locks.contains(&uuid) {
            bail!(ErrorKind::CircularReferenceFound);
        }

        let rc = {
            self.locks.insert(uuid);
            let mut file = self.archives.open(&path)?;
            let resource = T::load_from_file(self, file.as_mut())?;
            self.locks.remove(&uuid);

            Arc::new(RwLock::new(resource))
        };

        self.backends
            .index_mut::<T::Item>()
            .insert(&path, rc.clone())
            .unwrap();
        Ok(rc)
    }
}

struct ResourceBackends(pub Vec<Option<Box<Any>>>);

impl ResourceBackends {
    pub fn new() -> Self {
        ResourceBackends(Vec::new())
    }

    pub fn index_mut<T>(&mut self) -> &mut ResourceBackend<T>
        where T: Resource + ResourceIndex + 'static
    {
        self.0[T::type_index()]
            .as_mut()
            .expect("Tried to perform an operation on resource type that not registered.")
            .downcast_mut::<ResourceBackend<T>>()
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

        self.0[T::type_index()] = Some(Box::new(ResourceBackend::<T>::new()));
    }

    pub fn unload_unused(&mut self) {
        for v in &mut self.0 {
            if let &mut Some(ref mut backend) = v {
                backend
                    .downcast_mut::<Box<ITraitResourceBackend>>()
                    .unwrap()
                    .unload_unused();
            }
        }
    }
}

trait ITraitResourceBackend {
    fn unload_unused(&mut self);
}

impl<T> ITraitResourceBackend for ResourceBackend<T>
    where T: Resource + ResourceIndex + 'static
{
    fn unload_unused(&mut self) {
        self.unload_unused();
    }
}