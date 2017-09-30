use std::path::{Path, PathBuf};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::sync::{Arc, RwLock};

use rayon;
use futures::prelude::*;
use futures::sync::oneshot::{channel, Receiver};

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
///
/// Note that All of the io heavy operations will returns a `Future` wrapper instead of raw
/// bytes. These operations are executed on worker thread to avoid block main-thread.
pub struct FilesystemDriver {
    filesystems: HashMap<HashValue<str>, Arc<Box<Filesystem>>>,
    slave: rayon::ThreadPool,
}

impl FilesystemDriver {
    /// Create a new file-system driver. A worker thread will be spawned to perform
    /// io heavy operations.
    pub fn new() -> FilesystemDriver {
        let confs = rayon::Configuration::new().num_threads(1);
        FilesystemDriver {
            filesystems: HashMap::new(),
            slave: rayon::ThreadPool::new(confs).unwrap(),
        }
    }

    /// Mount a file-system drive with identifier.
    pub fn mount<S, F>(&mut self, ident: S, fs: F) -> Result<()>
        where S: Borrow<str>,
              F: Filesystem + 'static
    {
        let hash = HashValue::from(ident);

        if self.filesystems.get(&hash).is_some() {
            bail!(ErrorKind::DriveWithSameIdentFound);
        }

        self.filesystems.insert(hash, Arc::new(Box::new(fs)));
        Ok(())
    }

    /// Unmount a file-system from this collection.
    pub fn unmount<S>(&mut self, ident: S)
        where S: Borrow<str>
    {
        let hash = HashValue::from(ident);
        self.filesystems.remove(&hash);
    }

    /// Return whether the path points at an existing file.
    pub fn exists<S, P>(&self, ident: S, path: P) -> bool
        where S: Borrow<str>,
              P: AsRef<Path>
    {
        let hash = HashValue::from(ident);

        self.filesystems
            .get(&hash)
            .map(|fs| fs.exists(path.as_ref()))
            .unwrap_or(false)
    }

    /// Read all bytes until EOF in this source.
    pub fn load<S, P>(&self, ident: S, path: P) -> Result<DataFuture>
        where S: Borrow<str>,
              P: AsRef<Path> + Sync
    {
        let hash = HashValue::from(ident);
        if let Some(fs) = self.filesystems.get(&hash).map(|v| v.clone()) {
            let (tx, rx) = channel();
            self.slave
                .install(|| tx.send(load(fs, path.as_ref())))
                .unwrap();
            Ok(DataFuture(rx))
        } else {
            bail!(ErrorKind::DriveNotFound);
        }
    }
}

pub struct DataFuture(Receiver<Result<Vec<u8>>>);

impl Future for DataFuture {
    type Item = Vec<u8>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(Async::Ready(x)) => Ok(Async::Ready(x?)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => bail!(ErrorKind::FutureCanceled),
        }
    }
}

fn load(fs: Arc<Box<Filesystem>>, path: &Path) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    fs.load_into(path, &mut buf)?;
    Ok(buf)
}

/// Maps a local host directory into virtual file system.
pub struct DirectoryFS {
    wp: PathBuf,
}

impl DirectoryFS {
    /// Create a new disk filesystem.
    pub fn new<T>(path: T) -> Result<Self>
        where T: AsRef<Path>
    {
        let meta = fs::metadata(&path)?;
        if meta.is_dir() {
            Ok(DirectoryFS { wp: path.as_ref().to_owned() })
        } else {
            bail!(ErrorKind::NotFound);
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
        where T: AsRef<Path>
    {
        let file = fs::File::open(path)?;
        let archive = zip::ZipArchive::new(file)?;
        Ok(ZipFS { archive: RwLock::new(archive) })
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
            bail!(ErrorKind::NotFound);
        }
    }
}