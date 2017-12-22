use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;

use two_lock_queue;

use super::filesystem::{Filesystem, FilesystemDriver};
use super::errors::*;

/// The callbacks of async loader.
pub trait ResourceAsyncLoader: Send + Sync + 'static {
    fn on_finished(&mut self, _: &Path, _: Result<&[u8]>);
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

        let (tx, rx) = two_lock_queue::channel(1024);

        {
            let driver = driver.clone();
            thread::Builder::new()
                .name("RESOURCE".into())
                .spawn(|| { ResourceSystem::run(rx, driver); })
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
        where S: AsRef<str>,
              F: Filesystem + 'static
    {
        self.filesystems.write().unwrap().mount(ident, fs)
    }

    /// Unmount a file-system from this collection.
    #[inline]
    pub fn unmount<S>(&self, ident: S)
        where S: AsRef<str>
    {
        self.filesystems.write().unwrap().unmount(ident);
    }

    fn run(chan: two_lock_queue::Receiver<ResourceTask>, driver: Arc<RwLock<FilesystemDriver>>) {
        let mut buf = Vec::new();

        loop {
            match chan.recv().unwrap() {
                ResourceTask::Load { mut closure } => {
                    let driver = driver.read().unwrap();
                    closure(&driver, &mut buf);
                }

                ResourceTask::Stop => return,
            }
        }
    }

    fn load<T>(slave: &mut T, path: &Path, driver: &FilesystemDriver, buf: &mut Vec<u8>)
        where T: ResourceAsyncLoader
    {
        let from = buf.len();

        match driver.load_into(&path, buf) {
            Ok(_) => slave.on_finished(&path, Ok(&buf[from..])),
            Err(err) => slave.on_finished(&path, Err(err)),
        };
    }
}

/// The multi-thread friendly parts of `ResourceSystem`.
pub struct ResourceSystemShared {
    filesystems: Arc<RwLock<FilesystemDriver>>,
    chan: two_lock_queue::Sender<ResourceTask>,
}

enum ResourceTask {
    Load { closure: Box<FnMut(&FilesystemDriver, &mut Vec<u8>) + Send + Sync>, },
    Stop,
}

impl ResourceSystemShared {
    fn new(filesystems: Arc<RwLock<FilesystemDriver>>,
           chan: two_lock_queue::Sender<ResourceTask>)
           -> Self {
        ResourceSystemShared {
            filesystems: filesystems,
            chan: chan,
        }
    }

    /// Return whether the path points at an existing file.
    pub fn exists<T, P>(&self, path: P) -> bool
        where P: AsRef<Path>
    {
        self.filesystems.read().unwrap().exists(path)
    }

    /// Load a file at location `path` asynchronously.
    ///
    /// `ResourceAsyncLoader::on_finished` will be called if task finishs or any
    /// error triggered when loading.
    pub fn load_async<T, P>(&self, worker: T, path: P)
        where T: ResourceAsyncLoader,
              P: AsRef<Path>
    {
        // Hacks: Optimize this when Box<FnOnce> is usable.
        let path = path.as_ref().to_owned();
        let payload = Arc::new(RwLock::new(Some((worker, path))));
        let closure = move |d: &FilesystemDriver, b: &mut Vec<u8>| {
            // ..
            if let Some(mut data) = payload.write().unwrap().take() {
                ResourceSystem::load::<T>(&mut data.0, &data.1, d, b);
            }
        };

        self.chan
            .send(ResourceTask::Load { closure: Box::new(closure) })
            .unwrap();
    }
}

impl Drop for ResourceSystemShared {
    fn drop(&mut self) {
        self.chan.send(ResourceTask::Stop).unwrap();
    }
}