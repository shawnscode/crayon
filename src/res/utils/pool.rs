//! # ResourcePool
//!
//! The `ResourcePool` is a standardized resource manager that defines a set of interface for creation,
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
//! Everytime you create a resource at runtime, the `ResourcePool` will increases the reference count of
//! the resource by 1. And when you are done with the resource, its the user's responsibility to
//! drop the ownership of the resource. And when the last ownership to a given resource is dropped,
//! the corresponding resource is also destroyed.

use failure::Error;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::utils::prelude::{FastHashMap, HandleLike, ObjectPool};

use super::state::ResourceState;

pub trait ResourceLoader: Send + Sync {
    type Handle: Send;
    type Intermediate: Send;
    type Resource: Send;

    fn load(&self, _: Self::Handle, _: &[u8]) -> Result<Self::Intermediate, Error>;
    fn create(&self, _: Self::Handle, _: Self::Intermediate) -> Result<Self::Resource, Error>;
    fn delete(&self, _: Self::Handle, _: Self::Resource);
}

// The `ResourcePool` is a standardized resources manager that defines a set of interface for creation,
// destruction, sharing and lifetime management. It is used in all the built-in crayon modules.
pub struct ResourcePool<H, Loader>
where
    H: HandleLike + 'static,
    Loader: ResourceLoader<Handle = H> + Clone + 'static,
{
    items: ObjectPool<H, Item<Loader::Resource>>,
    requests: FastHashMap<H, Arc<Mutex<ResourceAsyncState<Loader::Intermediate>>>>,
    registry: FastHashMap<Uuid, H>,
    loader: Loader,
}

impl<H, Loader> ResourcePool<H, Loader>
where
    H: HandleLike + 'static,
    Loader: ResourceLoader<Handle = H> + Clone + 'static,
{
    /// Create a new and empty `ResourcePool`.
    pub fn new(loader: Loader) -> Self {
        ResourcePool {
            items: ObjectPool::new(),
            registry: FastHashMap::default(),
            requests: FastHashMap::default(),
            loader,
        }
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        let items = &mut self.items;
        let loader = &self.loader;

        self.requests.retain(|&handle, req| {
            let mut req = req.lock().unwrap();
            if let ResourceAsyncState::NotReady = *req {
                return true;
            }

            let mut tmp = ResourceAsyncState::NotReady;
            std::mem::swap(&mut *req, &mut tmp);

            match tmp {
                ResourceAsyncState::Err(err) => {
                    warn!("{:?}", err);
                    if let Some(item) = items.get_mut(handle) {
                        item.error = Some(err);
                    }
                }
                ResourceAsyncState::Ok(intermediate) => {
                    if let Some(item) = items.get_mut(handle) {
                        match loader.create(handle, intermediate) {
                            Ok(resource) => item.resource = Some(resource),
                            Err(err) => {
                                warn!("{:?}", err);
                                item.error = Some(err);
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }

            false
        });

        Ok(())
    }

    /// Create a resource with provided value instance.
    ///
    /// A associated `Handle` is returned.
    #[inline]
    pub fn create(&mut self, params: Loader::Intermediate) -> Result<H, Error> {
        let handle = self.alloc(None);
        match self.loader.create(handle, params) {
            Ok(value) => {
                self.items.get_mut(handle).unwrap().resource = Some(value);
                Ok(handle)
            }
            Err(error) => {
                self.delete(handle);
                Err(error)
            }
        }
    }

    /// Create a resource from file asynchronously.
    #[inline]
    pub fn create_from<T: AsRef<str>>(&mut self, url: T) -> Result<H, Error> {
        let url = url.as_ref();
        let uuid = crate::res::find(url)
            .ok_or_else(|| format_err!("Could not found resource '{}'.", url))?;
        self.create_from_uuid(uuid)
    }

    /// Create a named resource from file asynchronously.
    #[inline]
    pub fn create_from_uuid(&mut self, uuid: Uuid) -> Result<H, Error> {
        if let Some(&handle) = self.registry.get(&uuid) {
            self.items.get_mut(handle).unwrap().rc += 1;
            return Ok(handle);
        }

        let handle = self.alloc(Some(uuid));

        let rx = Arc::new(Mutex::new(ResourceAsyncState::NotReady));
        let tx = rx.clone();
        let loader = self.loader.clone();

        let result = crate::res::load_with_callback(uuid, move |rsp| match rsp {
            Ok(bytes) => {
                let itermediate = loader.load(handle, &bytes);

                match itermediate {
                    Ok(item) => {
                        *tx.lock().unwrap() = ResourceAsyncState::Ok(item);
                    }
                    Err(err) => {
                        *tx.lock().unwrap() = ResourceAsyncState::Err(err);
                    }
                }
            }

            Err(err) => {
                *tx.lock().unwrap() = ResourceAsyncState::Err(err);
            }
        });

        match result {
            Ok(_) => {
                self.requests.insert(handle, rx);
                Ok(handle)
            }
            Err(err) => {
                self.delete(handle);
                Err(err)
            }
        }
    }

    /// Deletes a resource from loadery.
    pub fn delete(&mut self, handle: H) {
        let disposed = self
            .items
            .get_mut(handle)
            .map(|e| {
                e.rc -= 1;
                e.rc == 0
            })
            .unwrap_or(false);

        if disposed {
            let e = self.items.free(handle).unwrap();

            if let Some(uuid) = e.uuid {
                self.registry.remove(&uuid);
            }

            if let Some(resource) = e.resource {
                self.loader.delete(handle, resource);
            }
        }
    }

    /// Get the resource state.
    #[inline]
    pub fn state(&self, handle: H) -> ResourceState {
        self.items
            .get(handle)
            .map(|e| {
                if e.resource.is_some() {
                    ResourceState::Ok
                } else if e.error.is_some() {
                    ResourceState::Err
                } else {
                    ResourceState::NotReady
                }
            })
            .unwrap_or(ResourceState::NotReady)
    }

    /// Checks if the handle is still avaiable in this pool.
    #[inline]
    pub fn contains(&self, handle: H) -> bool {
        self.items.contains(handle)
    }

    /// Return immutable reference to internal value with name `Handle`.
    #[inline]
    pub fn resource(&self, handle: H) -> Option<&Loader::Resource> {
        self.items.get(handle).and_then(|e| e.resource.as_ref())
    }

    /// Return mutable reference to internal value with name `Handle`.
    #[inline]
    pub fn resource_mut(&mut self, handle: H) -> Option<&mut Loader::Resource> {
        self.items.get_mut(handle).and_then(|e| e.resource.as_mut())
    }

    #[inline]
    fn alloc(&mut self, uuid: Option<Uuid>) -> H {
        let entry = Item {
            rc: 1,
            uuid,
            resource: None,
            error: None,
        };

        let handle = self.items.create(entry);

        if let Some(uuid) = uuid {
            self.registry.insert(uuid, handle);
        }

        handle
    }
}

struct Item<T> {
    rc: u32,
    uuid: Option<Uuid>,
    resource: Option<T>,
    error: Option<Error>,
}

enum ResourceAsyncState<T> {
    Ok(T),
    Err(Error),
    NotReady,
}
