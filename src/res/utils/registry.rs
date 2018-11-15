//! # Registry
//!
//! The `Registry` is a standardized resource manager that defines a set of interface for creation,
//! destruction, sharing and lifetime management. It is used in all the built-in crayon modules.
//!
//! ## Handle
//!
//! We are using a unique `Handle` object to represent a resource object safely. This approach
//! has several advantages, since it helps for saving state externally. E.G.:
//!
//! 1. It allows for the resource to be destroyed without leaving dangling pointers.
//! 2. Its perfectly safe to store and share the `Handle` even the underlying resource is
//! loading on the background thread.
//!
//! In some systems, actual resource objects are private and opaque, application will usually
//! not have direct access to a resource object in form of reference.
//!
//! ## Ownership & Lifetime
//!
//! For the sake of simplicity, the refenerce-counting technique is used for providing shared ownership
//! of a resource.
//!
//! Everytime you create a resource at runtime, the `Registry` will increases the reference count of
//! the resource by 1. And when you are done with the resource, its the user's responsibility to
//! drop the ownership of the resource. And when the last ownership to a given resource is dropped,
//! the corresponding resource is also destroyed.

use std::sync::{Arc, RwLock};
use uuid::Uuid;

use errors::*;
use utils::{FastHashMap, HandleLike, ObjectPool};

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
    payload: Arc<RwLock<Payload<H, R>>>,
    register: R,
}

impl<H: HandleLike + 'static, R: Register<Handle = H> + Clone + 'static> Registry<H, R> {
    /// Creates a new and empty `Registry`.
    pub fn new(register: R) -> Self {
        let payload = Payload {
            items: ObjectPool::new(),
            redirects: FastHashMap::default(),
        };

        Registry {
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
    pub fn create_from<T: AsRef<str>>(&self, url: T) -> Result<H> {
        let url = url.as_ref();

        let uuid = crate::res::find(url)
            .ok_or_else(|| format_err!("Could not found resource '{}'.", url))?;

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

        let payload = self.payload.clone();
        let register = self.register.clone();
        let cb = move |rsp: crate::res::request::Response| match rsp {
            Ok(bytes) => {
                let itermediate = register.load(handle, &bytes);
                let mut payload = payload.write().unwrap();
                let disposed = payload.items.get(handle).unwrap().rc <= 0;

                if disposed {
                    let entry = payload.items.free(handle).unwrap();

                    if let Some(uuid) = entry.uuid {
                        payload.redirects.remove(&uuid);
                    }

                    if let AsyncState::Ok(value) = entry.state {
                        register.detach(handle, value);
                    }
                } else {
                    match itermediate {
                        Ok(item) => match register.attach(handle, item) {
                            Ok(value) => {
                                payload.items.get_mut(handle).unwrap().state =
                                    AsyncState::Ok(value);
                            }
                            Err(err) => {
                                warn!("{:?}", err);
                                payload.items.get_mut(handle).unwrap().state = AsyncState::Err;
                            }
                        },
                        Err(err) => {
                            warn!("{:?}", err);
                            payload.items.get_mut(handle).unwrap().state = AsyncState::Err;
                        }
                    }
                }
            }
            Err(err) => {
                warn!("{:?}", err);
                let mut payload = payload.write().unwrap();
                if let Some(entry) = payload.items.get_mut(handle) {
                    entry.state = AsyncState::Err;
                }
            }
        };

        match crate::res::load_with_callback(uuid, cb) {
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

    /// Visits all key-value pairs in order.
    #[inline]
    pub fn iter<T: FnMut(H, &R::Value, u32, Option<Uuid>)>(&self, mut cb: T) {
        let payload = self.payload.read().unwrap();
        for (k, v) in payload.items.iter() {
            if let AsyncState::Ok(ref w) = v.state {
                cb(k, w, v.rc, v.uuid);
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
