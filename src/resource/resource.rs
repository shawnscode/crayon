//!

use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};
use std::thread;
use std::borrow::Borrow;
use std::time::Duration;

use deque;
use futures;

use utils::HashValue;
use super::{Ptr, Resource, ResourceParser};
use super::arena::ArenaWithCache;
use super::filesystem::{Filesystem, FilesystemDriver};
use super::errors::*;

pub type ResourceFuture<T> = futures::sync::oneshot::Receiver<Result<Ptr<T>>>;

pub struct ResourceSystemShared {
    filesystems: Arc<RwLock<FilesystemDriver>>,
    chan: deque::Worker<ResourceTask>,
}

enum ResourceTask {
    Request {
        id: TypeId,
        closure: Box<FnMut(&mut Any,
                           &mut FilesystemDriver,
                           &mut HashSet<HashValue<Path>>) + Send + Sync>,
    },
    UnloadUnused,
}

impl ResourceSystemShared {
    fn new(filesystems: Arc<RwLock<FilesystemDriver>>, chan: deque::Worker<ResourceTask>) -> Self {
        ResourceSystemShared {
            filesystems: filesystems,
            chan: chan,
        }
    }

    pub fn exists<T, P>(&self, path: P) -> bool
        where P: AsRef<Path>
    {
        self.filesystems.read().unwrap().exists(path)
    }

    pub fn load<T, P>(&self, path: P) -> ResourceFuture<T::Item>
        where T: ResourceParser,
              P: AsRef<Path>
    {
        let (tx, rx) = futures::sync::oneshot::channel();
        let path = path.as_ref().to_owned();

        // Hacks: Fix this when Box<FnOnce> is usable.
        let payload = Arc::new(RwLock::new(Some((path, tx))));
        let closure = move |mut a: &mut Any,
                            mut d: &mut FilesystemDriver,
                            mut l: &mut HashSet<HashValue<Path>>| {
            if let Some(data) = payload.write().unwrap().take() {
                let v = ResourceSystem::load::<T>(data.0, a, d, l);
                data.1.send(v).is_ok();
            }
        };

        self.chan
            .push(ResourceTask::Request {
                      id: TypeId::of::<T::Item>(),
                      closure: Box::new(closure),
                  });

        rx
    }

    /// Unload unused resources from memory.
    pub fn unload_unused(&mut self) {
        self.chan.push(ResourceTask::UnloadUnused);
    }
}

pub struct ResourceSystem {
    filesystems: Ptr<FilesystemDriver>,
    arenas: Ptr<HashMap<TypeId, Box<Any + Send + Sync>>>,
    shared: Arc<ResourceSystemShared>,
}

impl ResourceSystem {
    pub fn new() -> Result<Self> {
        let driver = Arc::new(RwLock::new(FilesystemDriver::new()));
        let arenas = Arc::new(RwLock::new(HashMap::new()));

        let (tx, rx) = deque::new();

        {
            let driver = driver.clone();
            let arenas = arenas.clone();

            thread::spawn(|| { ResourceSystem::run(rx, driver, arenas); });
        }

        let shared = ResourceSystemShared::new(driver.clone(), tx);

        Ok(ResourceSystem {
               filesystems: driver,
               arenas: arenas,
               shared: Arc::new(shared),
           })
    }

    pub fn shared(&self) -> Arc<ResourceSystemShared> {
        self.shared.clone()
    }

    /// Register a new resource type.
    #[inline]
    pub fn register<T>(&self)
        where T: Resource + Send + Sync + 'static
    {
        let id = TypeId::of::<T>();
        let mut arenas = self.arenas.write().unwrap();

        if !arenas.contains_key(&id) {
            let item = ArenaWithCache::<T>::with_capacity(0);
            arenas.insert(id, Box::new(item));
        }
    }

    /// Mount a file-system drive with identifier.
    #[inline]
    pub fn mount<S, F>(&self, ident: S, fs: F) -> Result<()>
        where S: Borrow<str>,
              F: Filesystem + 'static
    {
        self.filesystems.write().unwrap().mount(ident, fs)
    }

    /// Unmount a file-system from this collection.
    #[inline]
    pub fn unmount<S>(&self, ident: S)
        where S: Borrow<str>
    {
        self.filesystems.write().unwrap().unmount(ident);
    }

    fn run(stealer: deque::Stealer<ResourceTask>,
           driver: Ptr<FilesystemDriver>,
           arenas: Ptr<HashMap<TypeId, Box<Any + Send + Sync>>>) {
        let mut locks: HashSet<HashValue<Path>> = HashSet::new();
        loop {
            match stealer.steal() {
                deque::Stolen::Abort => continue,
                deque::Stolen::Empty => thread::sleep(Duration::from_millis(100)), 
                deque::Stolen::Data(task) => {
                    match task {
                        ResourceTask::Request { id, mut closure } => {
                            let mut driver = driver.write().unwrap();
                            let mut arenas = arenas.write().unwrap();
                            let mut arena =
                                arenas.get_mut(&id).expect("not registered resource type");

                            closure(arena.as_mut(), &mut driver, &mut locks);
                        }
                        ResourceTask::UnloadUnused => {
                            let mut arenas = arenas.write().unwrap();
                            for (_, v) in arenas.iter_mut() {
                                ResourceSystem::unload(v);
                            }
                        }   
                    }
                }
            }
        }
    }

    fn unload(arena: &mut Any) {
        arena.downcast_mut::<Box<Arena>>().unwrap().unload_unused();
    }

    fn load<T>(path: PathBuf,
               arena: &mut Any,
               driver: &mut FilesystemDriver,
               locks: &mut HashSet<HashValue<Path>>)
               -> Result<Ptr<T::Item>>
        where T: ResourceParser
    {
        let arena = arena.downcast_mut::<ArenaWithCache<T::Item>>().unwrap();

        let hash = (&path).into();
        if let Some(rc) = arena.get(hash) {
            return Ok(rc);
        }

        if locks.contains(&hash) {
            bail!(ErrorKind::CircularReferenceFound);
        }

        let rc = {
            locks.insert(hash);
            let bytes = driver.load(&path)?;
            let resource = T::parse(bytes)?;
            locks.remove(&hash);

            Arc::new(RwLock::new(resource))
        };

        arena.insert(hash, rc.clone());
        Ok(rc)
    }
}

/// Anonymous operations helper.
trait Arena {
    fn unload_unused(&mut self);
}

impl<T> Arena for ArenaWithCache<T>
    where T: Resource
{
    fn unload_unused(&mut self) {
        ArenaWithCache::unload_unused(self);
    }
}