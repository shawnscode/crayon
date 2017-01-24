use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Result, Error, ErrorKind};
use std::fmt::Debug;
use std::cell::RefCell;
use zip;

pub use std::io::{Read, Seek};

pub trait Archive: 'static + Debug {
    fn exists(&mut self, path: &str) -> bool;
    fn open<'a>(&'a mut self, path: &str) -> Result<Box<Read + 'a>>;
}

#[derive(Debug)]
pub struct FilesystemArchive {
    work_path: PathBuf,
}

impl FilesystemArchive {
    pub fn new<T>(path: T) -> Result<Self>
        where T: AsRef<Path>
    {
        let meta = fs::metadata(&path)?;
        if meta.is_dir() {
            Ok(FilesystemArchive { work_path: path.as_ref().to_owned() })
        } else {
            Err(Error::new(ErrorKind::NotFound, "can't find directory at given path."))
        }
    }

    pub fn open(&self, path: &str) -> Result<fs::File> {
        fs::File::open(self.work_path.join(path))
    }
}

impl Archive for FilesystemArchive {
    fn exists(&mut self, path: &str) -> bool {
        fs::metadata(self.work_path.join(path)).is_ok()
    }

    fn open<'a>(&'a mut self, path: &str) -> Result<Box<Read + 'a>> {
        let file = fs::File::open(self.work_path.join(path))?;
        Ok(Box::new(file))
    }
}

#[derive(Debug)]
pub struct ZipArchive {
    archive: zip::ZipArchive<fs::File>,
}

impl ZipArchive {
    pub fn new(file: fs::File) -> Result<Self> {
        let z = zip::ZipArchive::new(file)?;
        Ok(ZipArchive { archive: z })
    }
}

impl Archive for ZipArchive {
    fn exists(&mut self, path: &str) -> bool {
        self.archive.by_name(path).is_ok()
    }

    fn open<'a>(&'a mut self, path: &str) -> Result<Box<Read + 'a>> {
        let file = self.archive.by_name(path)?;
        Ok(Box::new(file))
    }
}

pub struct ArchiveCollection {
    archives: Vec<Box<RefCell<Archive>>>,
}

impl ArchiveCollection {
    pub fn new() -> Self {
        ArchiveCollection { archives: vec![] }
    }

    pub fn register<T>(&mut self, archive: T)
        where T: Archive
    {
        self.archives.push(Box::new(RefCell::new(archive)));
    }


    /// Returns true if the file is exists at specified path in registered
    /// archives.
    /// The search order is same as the order user register archives.
    pub fn exists<T>(&self, path: T) -> bool
        where T: AsRef<Path>
    {
        match path.as_ref().to_str() {
            Some(slice) => {
                for archive in &self.archives {
                    if archive.borrow_mut().exists(slice) {
                        return true;
                    }
                }
                false
            }
            None => false,
        }
    }

    /// Read all bytes until EOF in this source, placing them into buf.
    /// If successful, this function will return the total number of bytes read.
    ///
    /// # Errors
    ///
    /// If this function encounters an error of the kind ErrorKind::Interrupted then
    /// the error is ignored and the operation will continue.
    /// If any other read error is encountered then this function immediately returns.
    /// Any bytes which have already been read will be appended to buf.
    pub fn read<T>(&self, path: T, buf: &mut Vec<u8>) -> Result<usize>
        where T: AsRef<Path>
    {
        match path.as_ref().to_str() {
            Some(slice) => {
                for archive in &self.archives {
                    let mut v = archive.borrow_mut();
                    match v.open(slice).map(|mut t| t.read_to_end(buf)) {
                        Ok(result) => {
                            if result.is_ok() {
                                return result;
                            }
                        }
                        _ => continue,
                    };
                }
                Err(Error::new(ErrorKind::NotFound, "can't find file at given path."))
            }
            None => Err(Error::new(ErrorKind::InvalidInput, "path is not a valid unicode str.")),
        }
    }

    /// Read all bytes until EOF in this source, appending them into buf.
    /// If successful, this function returns the number of bytes which were read
    /// and appended to buf.
    ///
    /// # Errors
    ///
    /// If the data in this stream is not valid UTF-8 then an error is returned
    /// and buf is unchanged. See `read_to_end()` for other error semantics.
    pub fn read_to_string<T>(&self, path: T, buf: &mut String) -> Result<usize>
        where T: AsRef<Path>
    {
        unsafe {
            let len = buf.len();
            let mut vbuf = buf.as_mut_vec();

            match self.read(path, vbuf) {
                Ok(size) => {
                    if ::std::str::from_utf8(&vbuf[len..]).is_err() {
                        vbuf.set_len(len);
                        Err(Error::new(ErrorKind::InvalidData,
                                       "stream did not contain valid UTF-8."))
                    } else {
                        Ok(size)
                    }
                }
                other => {
                    vbuf.set_len(len);
                    other
                }
            }
        }
    }
}