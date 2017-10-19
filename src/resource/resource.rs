//!

use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};
use std::thread;
use std::borrow::Borrow;
use std::time::Duration;

use deque;
use futures;
use futures::prelude::*;

use utils::HashValue;
use super::{Ptr, Resource, ResourceParser};
use super::arena::{ArenaWithCache, ArenaInfo};
use super::filesystem::{Filesystem, FilesystemDriver};
use super::errors::*;

#[derive(Debug, Clone, Default)]
pub struct ResourceFrameInfo {
    pub arenas: HashMap<TypeId, ArenaInfo>,
}

pub struct ResourceSystem {
    filesystems: Ptr<FilesystemDriver>,
    arenas: Ptr<HashMap<TypeId, ArenaWrapper>>,
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
    pub fn register<T>(&self, size: usize)
        where T: Resource + Send + Sync + 'static
    {
        let id = TypeId::of::<T>();
        let mut arenas = self.arenas.write().unwrap();

        if !arenas.contains_key(&id) {
            let item = ArenaWithCache::<T>::with_capacity(size);
            arenas.insert(id, ArenaWrapper::new(item));
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

    ///
    pub fn advance(&self) -> Result<ResourceFrameInfo> {
        let mut info = ResourceFrameInfo { arenas: HashMap::new() };

        {
            let mut arenas = self.arenas.write().unwrap();
            for (id, v) in arenas.iter_mut() {
                let i = v.unload_unused();
                info.arenas.insert(*id, i);
            }
        }

        Ok(info)
    }

    fn run(stealer: deque::Stealer<ResourceTask>,
           driver: Ptr<FilesystemDriver>,
           arenas: Ptr<HashMap<TypeId, ArenaWrapper>>) {
        let mut locks: HashSet<HashValue<Path>> = HashSet::new();
        let mut buf = Vec::new();

        loop {
            match stealer.steal() {
                deque::Stolen::Abort => continue,
                deque::Stolen::Empty => thread::sleep(Duration::from_millis(100)), 
                deque::Stolen::Data(task) => {
                    match task {
                        ResourceTask::Request { id, mut closure } => {
                            let driver = driver.read().unwrap();
                            closure(&arenas, id, &driver, &mut locks, &mut buf);
                        }

                        ResourceTask::UnloadUnused => {
                            let mut arenas = arenas.write().unwrap();
                            for (_, v) in arenas.iter_mut() {
                                v.unload_unused();
                            }
                        }

                        ResourceTask::Stop => return,
                    }
                }
            }
        }
    }

    fn load<T>(path: &Path,
               arenas: &RwLock<HashMap<TypeId, ArenaWrapper>>,
               id: TypeId,
               driver: &FilesystemDriver,
               locks: &mut HashSet<HashValue<Path>>,
               buf: &mut Vec<u8>)
               -> Result<Ptr<T::Item>>
        where T: ResourceParser
    {
        let hash = (&path).into();

        {
            let mut arenas = arenas.write().unwrap();
            let v = arenas.get_mut(&id).ok_or(ErrorKind::NotRegistered)?;
            if let Some(rc) = ResourceSystem::get::<T>(v.arena.as_mut(), hash) {
                return Ok(rc);
            }
        }

        if locks.contains(&hash) {
            bail!(ErrorKind::CircularReferenceFound);
        }

        let rc = {
            locks.insert(hash);
            let from = buf.len();
            driver.load_into(&path, buf)?;
            let resource = T::parse(&buf[from..])?;
            locks.remove(&hash);
            Arc::new(RwLock::new(resource))
        };

        {
            let mut arenas = arenas.write().unwrap();
            let v = arenas.get_mut(&id).ok_or(ErrorKind::NotRegistered)?;
            ResourceSystem::insert::<T>(v.arena.as_mut(), hash, rc.clone());
        }

        Ok(rc)
    }

    #[inline]
    fn get<T>(arena: &mut Any, hash: HashValue<Path>) -> Option<Ptr<T::Item>>
        where T: ResourceParser
    {
        arena
            .downcast_mut::<ArenaWithCache<T::Item>>()
            .unwrap()
            .get(hash)
    }

    #[inline]
    fn insert<T>(arena: &mut Any, hash: HashValue<Path>, rc: Ptr<T::Item>)
        where T: ResourceParser
    {
        arena
            .downcast_mut::<ArenaWithCache<T::Item>>()
            .unwrap()
            .insert(hash, rc);
    }
}

pub struct ResourceFuture<T>(futures::sync::oneshot::Receiver<Result<Ptr<T>>>);

impl<T> Future for ResourceFuture<T>
    where T: Resource
{
    type Item = Ptr<T>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(Async::Ready(x)) => Ok(Async::Ready(x?)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => bail!(ErrorKind::FutureCanceled),
        }
    }
}

pub struct ResourceSystemShared {
    filesystems: Arc<RwLock<FilesystemDriver>>,
    chan: deque::Worker<ResourceTask>,
}

enum ResourceTask {
    Request {
        id: TypeId,
        closure: Box<FnMut(&RwLock<HashMap<TypeId, ArenaWrapper>>,
                           TypeId,
                           &FilesystemDriver,
                           &mut HashSet<HashValue<Path>>,
                           &mut Vec<u8>) + Send + Sync>,
    },
    UnloadUnused,
    Stop,
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

        // Hacks: Optimize this when Box<FnOnce> is usable.
        let payload = Arc::new(RwLock::new(Some((path, tx))));
        let closure = move |a: &RwLock<HashMap<TypeId, ArenaWrapper>>,
                            i: TypeId,
                            d: &FilesystemDriver,
                            mut l: &mut HashSet<HashValue<Path>>,
                            mut b: &mut Vec<u8>| {
            if let Some(data) = payload.write().unwrap().take() {
                let v = ResourceSystem::load::<T>(&data.0, a, i, d, l, b);
                data.1.send(v).is_ok();
            }
        };

        self.chan
            .push(ResourceTask::Request {
                      id: TypeId::of::<T::Item>(),
                      closure: Box::new(closure),
                  });

        ResourceFuture(rx)
    }

    /// Unload unused resources from memory.
    pub fn unload_unused(&self) {
        self.chan.push(ResourceTask::UnloadUnused);
    }
}

impl Drop for ResourceSystemShared {
    fn drop(&mut self) {
        self.chan.push(ResourceTask::Stop);
    }
}

/// Anonymous operations helper.
struct ArenaWrapper {
    arena: Box<Any + Send + Sync>,
    unload_unused: Box<FnMut(&mut Any) -> ArenaInfo + Send + Sync>,
}

impl ArenaWrapper {
    fn new<T>(item: ArenaWithCache<T>) -> Self
        where T: Resource + Send + Sync + 'static
    {
        let unload_unused = move |a: &mut Any| {
            let a = a.downcast_mut::<ArenaWithCache<T>>().unwrap();
            a.unload_unused();
            a.info()
        };

        ArenaWrapper {
            arena: Box::new(item),
            unload_unused: Box::new(unload_unused),
        }
    }

    #[inline]
    fn unload_unused(&mut self) -> ArenaInfo {
        (self.unload_unused)(self.arena.as_mut())
    }
}