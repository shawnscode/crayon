use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::sync::mpsc;

use super::filesystem::{Filesystem, FilesystemDriver};
use super::errors::*;

/// The callbacks of async loader.
pub trait ResourceAsyncLoader: Send + Sync + 'static {
    fn on_finished(self, _: &Path, _: Result<&[u8]>);
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

        // let (tx, rx) = two_lock_queue::channel(1024);
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
                    let driver = driver.read().unwrap();
                    task.execute(&driver, &mut buf);
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
    pub fn exists<T, P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.filesystems.read().unwrap().exists(path)
    }

    /// Load a file at location `path` asynchronously.
    ///
    /// `ResourceAsyncLoader::on_finished` will be called if task finishs or any
    /// error triggered when loading.
    pub fn load_async<T, P>(&self, loader: T, path: P)
    where
        T: ResourceAsyncLoader,
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
    fn execute(&mut self, _: &FilesystemDriver, _: &mut Vec<u8>);
}

struct TaskLoad<T>
where
    T: ResourceAsyncLoader,
{
    loader: Option<T>,
    path: PathBuf,
}

impl<T> Task for TaskLoad<T>
where
    T: ResourceAsyncLoader,
{
    fn execute(&mut self, driver: &FilesystemDriver, buf: &mut Vec<u8>) {
        if let Some(loader) = self.loader.take() {
            let from = buf.len();

            match driver.load_into(&self.path, buf) {
                Ok(_) => loader.on_finished(&self.path, Ok(&buf[from..])),
                Err(err) => loader.on_finished(&self.path, Err(err)),
            };
        }
    }
}
