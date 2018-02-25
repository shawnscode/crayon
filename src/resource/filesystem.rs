//! The virtual file-system module that allows user to load data asynchronously.

use std::path::{Component, Components, Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::sync::{Arc, RwLock};

use zip;

use utils::HashValue;
use super::errors::*;

/// `Filesystem` enumerates all the io operations that should be supported.
pub trait Filesystem: Sync + Send {
    /// Return whether the path points at an existing file.
    fn exists(&self, path: &Path) -> bool;

    /// Read all bytes until EOF in this source, and placing them into `buf`.
    fn load_into(&self, path: &Path, buf: &mut Vec<u8>) -> Result<()>;
}

/// The driver of the virtual filesystem (VFS).
#[derive(Default)]
pub struct FilesystemDriver {
    filesystems: HashMap<HashValue<str>, Arc<Box<Filesystem>>>,
    buf: Vec<u8>,
}

impl FilesystemDriver {
    /// Create a new file-system driver. A worker thread will be spawned to perform
    /// io heavy operations.
    pub fn new() -> FilesystemDriver {
        FilesystemDriver {
            filesystems: HashMap::new(),
            buf: Vec::new(),
        }
    }

    /// Mount a file-system drive with identifier.
    pub fn mount<S, F>(&mut self, ident: S, fs: F) -> Result<()>
    where
        S: AsRef<str>,
        F: Filesystem + 'static,
    {
        let hash = HashValue::from(ident);

        if self.filesystems.get(&hash).is_some() {
            return Err(Error::DriveIdentDuplicated);
        }

        self.filesystems.insert(hash, Arc::new(Box::new(fs)));
        Ok(())
    }

    /// Unmount a file-system from this collection.
    pub fn unmount<S>(&mut self, ident: S)
    where
        S: AsRef<str>,
    {
        let hash = HashValue::from(ident);
        self.filesystems.remove(&hash);
    }

    /// Return whether the path points at an existing file.
    pub fn exists<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        FilesystemDriver::parse(path.as_ref().components())
            .and_then(|(bundle, file)| {
                let hash = HashValue::from(bundle);
                let found = self.filesystems
                    .get(&hash)
                    .map(|fs| fs.exists(file))
                    .unwrap_or(false);
                Some(found)
            })
            .unwrap_or(false)
    }

    /// Read all bytes until EOF in this source.
    pub fn load_into<P>(&self, path: P, buf: &mut Vec<u8>) -> Result<&[u8]>
    where
        P: AsRef<Path> + Sync,
    {
        if let Some((bundle, file)) = FilesystemDriver::parse(path.as_ref().components()) {
            let hash = HashValue::from(bundle);
            if let Some(fs) = self.filesystems.get(&hash) {
                fs.load_into(file, buf)?;
                return Ok(&self.buf[..]);
            }

            Err(Error::DriveNotFound(bundle.into()))
        } else {
            Err(Error::DriveNotFound("".into()))
        }
    }

    fn parse(mut cmps: Components) -> Option<(&str, &Path)> {
        while let Some(v) = cmps.next() {
            if let Component::Normal(ident) = v {
                if let Some(ident) = ident.to_str() {
                    return Some((ident, cmps.as_path()));
                }
            }
        }

        None
    }
}

/// Maps a local host directory into virtual file system.
pub struct DirectoryFS {
    wp: PathBuf,
}

impl DirectoryFS {
    /// Create a new disk filesystem.
    pub fn new<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let meta = fs::metadata(&path)?;
        if meta.is_dir() {
            Ok(DirectoryFS {
                wp: path.as_ref().to_owned(),
            })
        } else {
            Err(Error::FilesystemNotFound(
                path.as_ref().to_str().unwrap_or("").into(),
            ))
        }
    }
}

impl Filesystem for DirectoryFS {
    fn exists(&self, path: &Path) -> bool {
        fs::metadata(self.wp.join(path)).is_ok()
    }

    fn load_into(&self, path: &Path, buf: &mut Vec<u8>) -> Result<()> {
        let mut file = fs::File::open(self.wp.join(path))?;
        file.read_to_end(buf)?;
        Ok(())
    }
}

/// A virtual file sytem that builds on a zip archive.
pub struct ZipFS {
    archive: RwLock<zip::ZipArchive<fs::File>>,
}

impl ZipFS {
    /// Create a new zip filesystem.
    pub fn new<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let file = fs::File::open(path)?;
        let archive = zip::ZipArchive::new(file)?;
        Ok(ZipFS {
            archive: RwLock::new(archive),
        })
    }
}

impl Filesystem for ZipFS {
    fn exists(&self, path: &Path) -> bool {
        path.to_str()
            .map(|name| self.archive.write().unwrap().by_name(name).is_ok())
            .unwrap_or(false)
    }

    fn load_into(&self, path: &Path, buf: &mut Vec<u8>) -> Result<()> {
        if let Some(name) = path.to_str() {
            let mut archive = self.archive.write().unwrap();
            let mut file = archive.by_name(name)?;
            file.read_to_end(buf)?;
            Ok(())
        } else {
            Err(Error::FileNotFound(path.to_str().unwrap_or("").into()))
        }
    }
}
