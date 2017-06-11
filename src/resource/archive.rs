use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::io::{Result, Error, ErrorKind};
use std::fmt::Debug;
use std::cell::RefCell;
use zip;

pub use std::io::{Read, Seek};

/// A reference to an open file from some kind of archive. An instance of a `File`
/// can be read and/or written depending on what options it was opened with.
pub trait File: Read + Seek + Debug {}

impl<T> File for T where T: Read + Seek + Debug {}

/// A archive layer that lets us define multiple "file systems" with various
/// backing stores, then merge them together into `ArchiveCollection`.
pub trait Archive: 'static + Debug {
    /// Check if the file or directory exists.
    fn exists(&self, path: &Path) -> bool;

    /// Attempts to open a file in read-only mode.
    fn open(&self, path: &Path) -> Result<Box<File>>;

    /// Returns the length of the file when uncompressed.
    fn len(&self, path: &Path) -> Result<u64>;
}

/// An archive backed by physical file system.
#[derive(Debug)]
pub struct FilesystemArchive {
    work_path: PathBuf,
}

impl FilesystemArchive {
    /// Returns a new filesystem archive at specified directory.
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
}

impl Archive for FilesystemArchive {
    fn exists(&self, path: &Path) -> bool {
        fs::metadata(self.work_path.join(path)).is_ok()
    }

    fn open(&self, path: &Path) -> Result<Box<File>> {
        let file = fs::File::open(self.work_path.join(path))?;
        Ok(Box::new(file))
    }

    fn len(&self, path: &Path) -> Result<u64> {
        fs::metadata(self.work_path.join(path)).map(|v| v.len())
    }
}

/// An archive backed by ZIP file.
#[derive(Debug)]
pub struct ZipArchive {
    archive: RefCell<zip::ZipArchive<fs::File>>,
}

impl ZipArchive {
    pub fn new(file: fs::File) -> Result<Self> {
        let z = zip::ZipArchive::new(file)?;
        Ok(ZipArchive { archive: RefCell::new(z) })
    }
}

/// A wrappre to contain a zip file.
#[derive(Debug)]
pub struct ZipFile {
    buffer: io::Cursor<Vec<u8>>,
}

impl ZipFile {
    fn new(file: &mut zip::read::ZipFile) -> Result<Self> {
        let mut b = Vec::new();
        file.read_to_end(&mut b)?;
        Ok(ZipFile { buffer: io::Cursor::new(b) })
    }
}

impl io::Read for ZipFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.buffer.read(buf)
    }
}

impl io::Seek for ZipFile {
    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64> {
        self.buffer.seek(pos)
    }
}

impl Archive for ZipArchive {
    fn exists(&self, path: &Path) -> bool {
        if let Some(str_path) = path.to_str() {
            self.archive.borrow_mut().by_name(str_path).is_ok()
        } else {
            false
        }
    }

    fn open(&self, path: &Path) -> Result<Box<File>> {
        if let Some(str_path) = path.to_str() {
            let mut borrowed = self.archive.borrow_mut();
            let zip = ZipFile::new(&mut borrowed.by_name(str_path)?)?;
            Ok(Box::new(zip))
        } else {
            bail!(ErrorKind::NotFound);
        }
    }

    fn len(&self, path: &Path) -> Result<u64> {
        if let Some(str_path) = path.to_str() {
            let mut borrowed = self.archive.borrow_mut();
            Ok((borrowed.by_name(str_path)?).size())
        } else {
            bail!(ErrorKind::NotFound);
        }
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
        for archive in &self.archives {
            if archive.borrow_mut().exists(path.as_ref()) {
                return true;
            }
        }
        false
    }

    /// Attempts to open a file in read-only mode.
    pub fn open<T>(&self, path: T) -> Result<Box<File>>
        where T: AsRef<Path>
    {
        for archive in &self.archives {
            let v = archive.borrow_mut();
            match v.open(path.as_ref()) {
                Ok(file) => return Ok(file),
                _ => continue,
            };
        }
        bail!(ErrorKind::NotFound);
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
        for archive in &self.archives {
            let v = archive.borrow_mut();
            match v.open(path.as_ref()).map(|mut t| t.read_to_end(buf)) {
                Ok(result) => {
                    if result.is_ok() {
                        return result;
                    }
                }
                _ => continue,
            };
        }
        bail!(ErrorKind::NotFound);
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