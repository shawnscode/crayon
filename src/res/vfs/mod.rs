pub mod disk;

pub use self::disk::DiskFS;

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;

use utils::hash_value::HashValue;

use super::errors::*;

pub trait VFS: Send + Sync {
    /// Opens a readable file at location.
    fn read(&self, location: &Path) -> Result<Box<Read + Send>>;

    // /// Retrieves all file and directory entries in the given directory.
    // fn read_dir(&self, location: &Path) -> Result<Box<Iterator<Item = PathBuf>>>;

    /// Checks whether or not it is a directory.
    fn is_dir(&self, location: &Path) -> bool;

    /// Checks if the file exists.
    fn exists(&self, location: &Path) -> bool;

    /// Returns true if the file has been modified since `ts`.
    fn modified_since(&self, location: &Path, ts: SystemTime) -> bool;
}

pub struct VFSDriver {
    mounts: HashMap<HashValue<str>, Box<VFS>>,
}

impl VFSDriver {
    /// Create a new file-system driver.
    pub fn new() -> Self {
        VFSDriver {
            mounts: HashMap::new(),
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
            return Err(Error::VFS(format!("{:?} has been mounted.", hash)));
        }

        self.mounts.insert(hash, Box::new(vfs));
        Ok(())
    }

    pub fn read<T>(&self, fs: T, file: &Path) -> Result<Box<Read + Send>>
    where
        T: Into<HashValue<str>>,
    {
        let fs = fs.into();
        if let Some(vfs) = self.mounts.get(&fs) {
            vfs.read(file)
        } else {
            Err(Error::VFS(format!("Undefined VFS {:?}.", fs)))
        }
    }

    pub fn is_dir<T>(&self, fs: T, file: &Path) -> Result<bool>
    where
        T: Into<HashValue<str>>,
    {
        let fs = fs.into();
        if let Some(vfs) = self.mounts.get(&fs.into()) {
            Ok(vfs.is_dir(file))
        } else {
            Err(Error::VFS(format!("Undefined VFS {:?}.", fs)))
        }
    }

    pub fn exists<T>(&self, fs: T, file: &Path) -> Result<bool>
    where
        T: Into<HashValue<str>>,
    {
        let fs = fs.into();
        if let Some(vfs) = self.mounts.get(&fs.into()) {
            Ok(vfs.exists(file))
        } else {
            Err(Error::VFS(format!("Undefined VFS {:?}.", fs)))
        }
    }

    pub fn modified_since<T>(&self, fs: T, file: &Path, ts: SystemTime) -> Result<bool>
    where
        T: Into<HashValue<str>>,
    {
        let fs = fs.into();
        if let Some(vfs) = self.mounts.get(&fs.into()) {
            Ok(vfs.modified_since(file, ts))
        } else {
            Err(Error::VFS(format!("Undefined VFS {:?}.", fs)))
        }
    }
}
