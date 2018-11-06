use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use errors::*;

use super::VFS;

pub struct Directory {
    root: PathBuf,
}

impl Directory {
    pub fn new<T: Into<PathBuf>>(root: T) -> Result<Self> {
        let root = root.into();
        info!("Creates directory based virtual file system at {:?}.", root);

        let metadata = fs::metadata(&root)?;
        if metadata.is_dir() {
            Ok(Directory { root: root })
        } else {
            bail!("Disk file-system must be associated with a readable directory.");
        }
    }
}

impl VFS for Directory {
    fn read_to_end(&self, location: &Path, mut buf: &mut Vec<u8>) -> Result<usize> {
        let location = self.root.join(location);
        let mut file = fs::File::open(&location)?;
        let len = file.read_to_end(&mut buf)?;
        Ok(len)
    }

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
