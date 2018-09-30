//! The `Registry` is a standardized resources manager that defines a set of interface for creation,
//! destruction, sharing and lifetime management. It is used in all the built-in crayon modules.

use std::sync::{Arc, RwLock};
use uuid::Uuid;

use errors::*;
use utils::{FastHashMap, HandleLike, ObjectPool};

use super::{Loader, Location, ResourceSystemShared};

pub trait Register: Send + Sync {
    type Handle: Send + Sync;
    type Intermediate;
    type Value: Send + Sync;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate>;
    fn attach(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Value>;
    fn detach(&self, handle: Self::Handle, value: Self::Value);
}

// The `Registry` is a standardized resources manager that defines a set of interface for creation,
// destruction, sharing and lifetime management. It is used in all the built-in crayon modules.
pub struct Registry<H: HandleLike + 'static, R: Register<Handle = H> + Clone + 'static> {
    res: Arc<ResourceSystemShared>,
    payload: Arc<RwLock<Payload<H, R>>>,
    register: R,
}

impl<H: HandleLike + 'static, R: Register<Handle = H> + Clone + 'static> Registry<H, R> {
    /// Creates a new and empty `Registry`.
    pub fn new(res: Arc<ResourceSystemShared>, register: R) -> Self {
        let payload = Payload {
            items: ObjectPool::new(),
            redirects: FastHashMap::default(),
        };

        Registry {
            res: res,
            payload: Arc::new(RwLock::new(payload)),
            register: register,
        }
    }

    /// Creates a resource with provided value instance.
    ///
    /// A associated `Handle` is returned.
    pub fn create(&self, params: R::Intermediate) -> Result<H> {
        let entry = Entry {
            rc: 1,
            uuid: None,
            state: AsyncState::NotReady,
        };

        let mut payload = self.payload.write().unwrap();

        let handle = payload.items.create(entry);
        let value = self.register.attach(handle, params)?;
        payload.items.get_mut(handle).unwrap().state = AsyncState::Ok(value);

        Ok(handle)
    }

    /// Creates a resource from readable location.
    pub fn create_from<'a, T>(&'a self, location: T) -> Result<H>
    where
        T: Into<Location<'a>>,
    {
        let location = location.into();

        let uuid = self.res.redirect(location).ok_or_else(|| {
            format_err!(
                "Could NOT found resource '{}' at '{}'.",
                location.filename(),
                location.vfs()
            )
        })?;

        self.create_from_uuid(uuid)
    }

    /// Creates a resource from Uuid.
    pub fn create_from_uuid(&self, uuid: Uuid) -> Result<H> {
        let handle = {
            let mut payload = self.payload.write().unwrap();

            if let Some(&handle) = payload.redirects.get(&uuid) {
                payload.items.get_mut(handle).unwrap().rc += 1;
                return Ok(handle);
            }

            let entry = Entry {
                rc: 1,
                uuid: Some(uuid),
                state: AsyncState::NotReady,
            };

            let handle = payload.items.create(entry);
            payload.redirects.insert(uuid, handle);
            handle
        };

        let loader = RegistryLoader {
            handle: handle,
            register: self.register.clone(),
            payload: self.payload.clone(),
        };

        match self.res.load_from_uuid(loader, uuid) {
            Err(err) => {
                self.payload.write().unwrap().items.free(handle).unwrap();
                return Err(err);
            }
            _ => {}
        }

        Ok(handle)
    }

    /// Deletes a resource from registery.
    pub fn delete(&self, handle: H) {
        let mut payload = self.payload.write().unwrap();

        let disposed = payload
            .items
            .get_mut(handle)
            .map(|entry| {
                entry.rc -= 1;
                match entry.state {
                    AsyncState::Ok(_) | AsyncState::Err => entry.rc == 0,
                    _ => false,
                }
            }).unwrap_or(false);

        if disposed {
            let entry = payload.items.free(handle).unwrap();

            if let Some(uuid) = entry.uuid {
                payload.redirects.remove(&uuid);
            }

            if let AsyncState::Ok(value) = entry.state {
                self.register.detach(handle, value);
            }
        }
    }

    /// Gets the underlying `uuid` of handle.
    #[inline]
    pub fn uuid(&self, handle: H) -> Option<Uuid> {
        self.payload
            .read()
            .unwrap()
            .items
            .get(handle)
            .and_then(|v| v.uuid)
    }

    /// Blocks current thread until the loading process of resource finished.
    pub fn wait_until(&self, handle: H) -> Result<()> {
        if let Some(uuid) = self.uuid(handle) {
            self.res.wait_until(uuid)
        } else {
            Ok(())
        }
    }

    /// Blocks current thread until the loading process of resource finished.
    pub fn wait_and<F: FnOnce(&R::Value) -> T, T>(&self, handle: H, map: F) -> Option<T> {
        if let Some(uuid) = self.uuid(handle) {
            if self.res.wait_until(uuid).is_ok() {
                return self.get(handle, map);
            }
        }

        None
    }

    /// Gets the length of this `Registry`.
    #[inline]
    pub fn len(&self) -> usize {
        self.payload.read().unwrap().items.len()
    }

    /// Returns true if the `Registry` contains a resource associated with `handle`.
    #[inline]
    pub fn contains(&self, handle: H) -> bool {
        self.payload.read().unwrap().items.contains(handle)
    }

    /// Gets the registery value if available.
    #[inline]
    pub fn get<F: FnOnce(&R::Value) -> T, T>(&self, handle: H, map: F) -> Option<T> {
        self.payload
            .read()
            .unwrap()
            .items
            .get(handle)
            .and_then(|v| match v.state {
                AsyncState::Ok(ref value) => Some(value),
                _ => None,
            }).map(|v| map(v))
    }
}

struct Payload<H: HandleLike, R: Register<Handle = H>> {
    items: ObjectPool<H, Entry<R::Value>>,
    redirects: FastHashMap<Uuid, H>,
}

enum AsyncState<T> {
    Ok(T),
    Err,
    NotReady,
}

struct Entry<T> {
    rc: u32,
    uuid: Option<Uuid>,
    state: AsyncState<T>,
}

struct RegistryLoader<H: HandleLike, R: Register<Handle = H>> {
    handle: H,
    register: R,
    payload: Arc<RwLock<Payload<H, R>>>,
}

impl<H: HandleLike + 'static, R: Register<Handle = H> + 'static> Loader for RegistryLoader<H, R> {
    fn load(&self, bytes: &[u8]) -> Result<()> {
        let rsp = self.register.load(self.handle, bytes);

        {
            let mut payload = self.payload.write().unwrap();
            let disposed = payload.items.get(self.handle).unwrap().rc <= 0;

            if disposed {
                let entry = payload.items.free(self.handle).unwrap();

                if let Some(uuid) = entry.uuid {
                    payload.redirects.remove(&uuid);
                }

                if let AsyncState::Ok(value) = entry.state {
                    self.register.detach(self.handle, value);
                }
            } else {
                match rsp {
                    Ok(item) => match self.register.attach(self.handle, item) {
                        Ok(value) => {
                            payload.items.get_mut(self.handle).unwrap().state =
                                AsyncState::Ok(value);
                        }
                        Err(err) => {
                            warn!("{:?}", err);
                            payload.items.get_mut(self.handle).unwrap().state = AsyncState::Err;
                            return Err(err);
                        }
                    },
                    Err(err) => {
                        warn!("{:?}", err);
                        payload.items.get_mut(self.handle).unwrap().state = AsyncState::Err;
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }
}
