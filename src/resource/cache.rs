use std::any::Any;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use utility::HandleSet;
use super::{ResourceHandle, Resource};
use super::errors::*;
use super::archive::ArchiveCollection;

pub struct ResourceDesc {
    rc: usize,
    resource: Box<Any>,
}

pub struct Cache {
    archives: ArchiveCollection,
    names: HashMap<PathBuf, ResourceHandle>,
    loads: Vec<Option<ResourceDesc>>,
    handles: HandleSet,
    buf: Vec<u8>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            archives: ArchiveCollection::new(),
            names: HashMap::new(),
            loads: Vec::new(),
            handles: HandleSet::new(),
            buf: Vec::new(),
        }
    }

    pub fn archives(&self) -> &ArchiveCollection {
        &self.archives
    }

    pub fn archives_mut(&mut self) -> &mut ArchiveCollection {
        &mut self.archives
    }

    pub fn load<T, U>(&mut self, path: U) -> Result<ResourceHandle>
        where U: AsRef<Path>,
              T: Resource + 'static
    {
        let path = path.as_ref();
        if let Some(&handle) = self.names.get(path) {
            let mut v = &mut self.loads
                .get_mut(handle.index() as usize)
                .unwrap()
                .as_mut()
                .unwrap();
            v.rc += 1;
            Ok(handle)
        } else {
            let handle = self.handles.create().into();

            self.buf.clear();
            self.archives.read(&path, &mut self.buf)?;
            let desc = ResourceDesc {
                rc: 1,
                resource: Box::new(T::from_bytes(self.buf.as_slice())?),
            };

            self.names.insert(path.to_owned(), handle);
            self.loads[handle.index() as usize] = Some(desc);
            Ok(handle)
        }
    }

    pub fn unload<U>(&mut self, path: U)
        where U: AsRef<Path>
    {
        let path = path.as_ref();

        if let Some(&handle) = self.names.get(path) {
            let clean = {
                let mut v =
                    &mut self.loads.get_mut(handle.index() as usize).unwrap().as_mut().unwrap();
                v.rc -= 1;
                v.rc == 0
            };

            if clean {
                self.names.remove(path);
                self.loads[handle.index() as usize] = None;
            }
        }
    }

    #[inline]
    pub fn get<T>(&self, handle: ResourceHandle) -> Option<&T>
        where T: Resource + 'static
    {
        if self.handles.is_alive(*handle) {
            if let Some(v) = self.loads[handle.index() as usize]
                .as_ref()
                .unwrap()
                .resource
                .downcast_ref::<T>() {
                return Some(&v);
            }
        }

        None
    }

    // #[inline]
    // pub fn get_mut<T>(&mut self, handle: ResourceHandle) -> Option<&mut T>
    //     where T: Resource + 'static
    // {
    //     if self.handles.is_alive(*handle) {
    //         if let Some(v) = self.loads[handle.index() as usize]
    //             .as_mut()
    //             .unwrap()
    //             .resource
    //             .downcast_mut::<T>() {
    //             return Some(&mut v);
    //         }
    //     }

    //     None
    // }
}