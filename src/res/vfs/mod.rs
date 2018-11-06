pub mod dir;
pub use self::dir::Directory;

pub mod manifest;
pub use self::manifest::Manifest;

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use uuid::Uuid;

use errors::*;
use utils::{FastHashMap, HashValue};

pub trait VFS: Send + Sync + 'static {
    /// Opens a readable file at location.
    fn read_to_end(&self, location: &Path, buf: &mut Vec<u8>) -> Result<usize>;

    /// Checks whether or not it is a directory.
    fn is_dir(&self, location: &Path) -> bool;

    /// Checks if the file exists.
    fn exists(&self, location: &Path) -> bool;

    /// Returns true if the file has been modified since `ts`.
    fn modified_since(&self, location: &Path, ts: SystemTime) -> bool;
}

pub struct VFSInstance {
    vfs: Box<dyn VFS>,
    manifest: Manifest,
}

impl VFSInstance {
    pub fn new<T: VFS>(vfs: T) -> Result<Self> {
        let mut buf = Vec::new();
        vfs.read_to_end(manifest::NAME.as_ref(), &mut buf)?;

        let instance = VFSInstance {
            vfs: Box::new(vfs),
            manifest: manifest::Manifest::load_from(&mut Cursor::new(&buf))?,
        };

        Ok(instance)
    }

    #[inline]
    pub fn redirect<T>(&self, filename: T) -> Option<Uuid>
    where
        T: AsRef<str>,
    {
        self.manifest.redirect(filename)
    }

    #[inline]
    pub fn locate(&self, uuid: Uuid) -> Option<PathBuf> {
        self.manifest.locate(uuid)
    }

    #[inline]
    pub fn contains(&self, uuid: Uuid) -> bool {
        self.manifest.contains(uuid)
    }
}

impl VFS for VFSInstance {
    #[inline]
    fn read_to_end(&self, location: &Path, buf: &mut Vec<u8>) -> Result<usize> {
        self.vfs.read_to_end(location, buf)
    }

    #[inline]
    fn is_dir(&self, location: &Path) -> bool {
        self.vfs.is_dir(location)
    }

    #[inline]
    fn exists(&self, location: &Path) -> bool {
        self.vfs.exists(location)
    }

    #[inline]
    fn modified_since(&self, location: &Path, ts: SystemTime) -> bool {
        self.vfs.modified_since(location, ts)
    }
}

pub struct VFSDriver {
    mounts: FastHashMap<HashValue<str>, Arc<VFSInstance>>,
}

impl VFSDriver {
    /// Create a new file-system driver.
    pub fn new() -> Self {
        VFSDriver {
            mounts: FastHashMap::default(),
        }
    }

    /// Mount a file-system drive with identifier.
    pub fn mount<T, F>(&mut self, name: T, vfs: F) -> Result<()>
    where
        T: Into<HashValue<str>>,
        F: VFS + 'static,
    {
        let hash = name.into();
        if self.mounts.get(&hash).is_some() {
            bail!(
                "Virtual file system with identifier {:?} has been mounted already.",
                hash
            );
        }

        self.mounts.insert(hash, Arc::new(VFSInstance::new(vfs)?));
        Ok(())
    }

    /// Gets vfs instance which contains resource with `uuid`.
    pub fn vfs_from_uuid(&self, uuid: Uuid) -> Option<Arc<VFSInstance>> {
        for v in self.mounts.values() {
            if v.contains(uuid) {
                return Some(v.clone());
            }
        }

        None
    }

    /// Gets vfs with specified identifier `fs`.
    pub fn vfs<T>(&self, fs: T) -> Option<Arc<VFSInstance>>
    where
        T: Into<HashValue<str>>,
    {
        self.mounts.get(&fs.into()).map(|vfs| vfs.clone())
    }
}
