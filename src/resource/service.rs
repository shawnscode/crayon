use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::sync::mpsc;

use super::filesystem::{Filesystem, FilesystemDriver};
use super::errors::*;

/// A buffered filesystem driver.
pub struct ResourceFS<'a> {
    buf: &'a mut Vec<u8>,
    driver: &'a FilesystemDriver,
}

impl<'a> ResourceFS<'a> {
    /// Return whether the path points at an existing file.
    #[inline]
    pub fn exists<P>(&self, path: P) -> bool
    where
        P: AsRef<Path> + Sync,
    {
        self.driver.exists(path)
    }

    /// Read all bytes until EOF in this source.
    #[inline]
    pub fn load<P>(&mut self, path: P) -> Result<&[u8]>
    where
        P: AsRef<Path> + Sync,
    {
        self.driver.load_into(path, &mut self.buf)?;
        Ok(&self.buf[..])
    }
}

/// The executor of async resource task.
pub trait ResourceTask: Send + 'static {
    fn execute(self, _: &mut ResourceFS, _: &Path);
}

/// Takes care of loading data asynchronously through pluggable filesystems.
pub struct ResourceSystem {
    filesystems: Arc<RwLock<FilesystemDriver>>,
    shared: Arc<ResourceSystemShared>,
}

impl ResourceSystem {
    /// Creates a new `ResourceSystem`.
    ///
    /// Notes that this will spawn a worker thread running background to perform
    /// io requests.
    pub fn new() -> Result<Self> {
        let driver = Arc::new(RwLock::new(FilesystemDriver::new()));
        let (tx, rx) = mpsc::channel();

        {
            let driver = driver.clone();
            thread::Builder::new()
                .name("RESOURCE".into())
                .spawn(move || {
                    ResourceSystem::run(&rx, &driver);
                })
                .unwrap();
        }

        let shared = ResourceSystemShared::new(driver.clone(), tx);

        Ok(ResourceSystem {
            filesystems: driver,
            shared: Arc::new(shared),
        })
    }

    /// Returns the shared parts of `ResourceSystem`.
    pub fn shared(&self) -> Arc<ResourceSystemShared> {
        self.shared.clone()
    }

    /// Mount a file-system drive with identifier.
    #[inline]
    pub fn mount<S, F>(&self, ident: S, fs: F) -> Result<()>
    where
        S: AsRef<str>,
        F: Filesystem + 'static,
    {
        self.filesystems.write().unwrap().mount(ident, fs)
    }

    /// Unmount a file-system from this collection.
    #[inline]
    pub fn unmount<S>(&self, ident: S)
    where
        S: AsRef<str>,
    {
        self.filesystems.write().unwrap().unmount(ident);
    }

    fn run(chan: &mpsc::Receiver<Command>, driver: &RwLock<FilesystemDriver>) {
        let mut buf = Vec::new();
        loop {
            match chan.recv().unwrap() {
                Command::Task(mut task) => {
                    buf.clear();

                    let driver = driver.read().unwrap();
                    let mut fs = ResourceFS {
                        buf: &mut buf,
                        driver: &driver,
                    };

                    task.execute(&mut fs);
                }

                Command::Stop => return,
            }
        }
    }
}

/// The multi-thread friendly parts of `ResourceSystem`.
pub struct ResourceSystemShared {
    filesystems: Arc<RwLock<FilesystemDriver>>,
    chan: Mutex<mpsc::Sender<Command>>,
}

impl ResourceSystemShared {
    fn new(filesystems: Arc<RwLock<FilesystemDriver>>, chan: mpsc::Sender<Command>) -> Self {
        ResourceSystemShared {
            filesystems: filesystems,
            chan: Mutex::new(chan),
        }
    }

    /// Return whether the path points at an existing file.
    pub fn exists<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.filesystems.read().unwrap().exists(path)
    }

    /// Load a file at location `path` asynchronously.
    ///
    /// `ResourceTask::execute` will be called if task finishs or any
    /// error triggered when loading.
    pub fn load_async<'a, T, P>(&'a self, loader: T, path: P)
    where
        T: ResourceTask + 'static,
        P: Into<PathBuf>,
    {
        let task = TaskLoad {
            loader: Some(loader),
            path: path.into(),
        };

        self.chan
            .lock()
            .unwrap()
            .send(Command::Task(Box::new(task)))
            .unwrap();
    }
}

impl Drop for ResourceSystemShared {
    fn drop(&mut self) {
        self.chan.lock().unwrap().send(Command::Stop).unwrap();
    }
}

enum Command {
    Task(Box<Task>),
    Stop,
}

trait Task: Send {
    fn execute(&mut self, _: &mut ResourceFS);
}

struct TaskLoad<T>
where
    T: ResourceTask,
{
    loader: Option<T>,
    path: PathBuf,
}

impl<T> Task for TaskLoad<T>
where
    T: ResourceTask,
{
    fn execute(&mut self, fs: &mut ResourceFS) {
        if let Some(loader) = self.loader.take() {
            loader.execute(fs, &self.path);
        }
    }
}
