use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::super::errors::*;
use super::VFS;

pub struct DiskFS {
    root: PathBuf,
}

impl DiskFS {
    pub fn new<T: Into<PathBuf>>(root: T) -> Result<Self> {
        let root = root.into();

        let metadata = fs::metadata(&root)?;
        if metadata.is_dir() {
            Ok(DiskFS { root: root })
        } else {
            Err(Error::VFS(format!("{:?} is not a decent directory.", root)))
        }
    }
}

impl VFS for DiskFS {
    fn read(&self, location: &Path) -> Result<Box<Read + Send>> {
        let location = self.root.join(location);
        let file = fs::File::open(&location).map_err(|_| Error::FileNotFound(location))?;
        Ok(Box::new(file))
    }

    // fn read_dir(&self, location: &Path) -> Result<Box<Iterator<Item = PathBuf>>> {}
    fn is_dir(&self, location: &Path) -> bool {
        self.root.join(location).is_dir()
    }

    fn exists(&self, location: &Path) -> bool {
        self.root.join(location).exists()
    }

    fn modified_since(&self, location: &Path, ts: SystemTime) -> bool {
        let metadata = self.root.join(location).metadata().unwrap();
        ts == metadata.modified().unwrap()
    }
}
