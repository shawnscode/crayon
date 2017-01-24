use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Read, Result, Error, ErrorKind};
use std::fmt::Debug;

pub trait Archive: 'static + Debug {
    fn exists(&self, path: &str) -> bool;
    fn open(&self, path: &str) -> Result<Box<Read>>;
}

#[derive(Debug)]
pub struct FilesystemArchive {
    work_path: PathBuf,
}

impl FilesystemArchive {
    pub fn new<T>(path: T) -> Self
        where T: AsRef<Path>
    {
        FilesystemArchive { work_path: path.as_ref().to_owned() }
    }
}

impl Archive for FilesystemArchive {
    fn exists(&self, path: &str) -> bool {
        fs::metadata(self.work_path.join(path)).is_ok()
    }

    fn open(&self, path: &str) -> Result<Box<Read>> {
        let file = fs::File::open(self.work_path.join(path))?;
        Ok(Box::new(file))
    }
}

pub struct ArchiveCollection {
    archives: Vec<Box<Archive>>,
}

impl ArchiveCollection {
    pub fn new() -> Self {
        ArchiveCollection { archives: vec![] }
    }

    pub fn register<T>(&mut self, archive: T)
        where T: Archive
    {
        self.archives.push(Box::new(archive));
    }

    pub fn exists<T>(&self, path: T) -> bool
        where T: AsRef<Path>
    {
        match path.as_ref().to_str() {
            Some(slice) => {
                for archive in &self.archives {
                    if archive.exists(slice) {
                        return true;
                    }
                }
                false
            }
            None => false,
        }
    }

    pub fn open<T>(&self, path: T) -> Result<Box<Read>>
        where T: AsRef<Path>
    {
        match path.as_ref().to_str() {
            Some(slice) => {
                for archive in &self.archives {
                    match archive.open(slice) {
                        Ok(result) => return Ok(result),
                        _ => continue,
                    }
                }
                Err(Error::new(ErrorKind::NotFound, "can't find file at given path."))
            }
            None => Err(Error::new(ErrorKind::InvalidInput, "path is not a valid unicode str.")),
        }
    }
}