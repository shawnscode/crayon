//! The `Registry` is a standardized resources manager that defines a set of interface for creation,
//! destruction, sharing and lifetime management. It is used in all the built-in crayon modules.

use std::sync::Arc;
use uuid::Uuid;

use errors::*;
use utils::{FastHashMap, Handle, ObjectPool};

use super::{Loader, Location, Promise, ResourceSystemShared};

enum AsyncState<T> {
    Ok(T),
    NotReady(Arc<Promise>),
}

struct Entry<T> {
    rc: u32,
    uuid: Option<Uuid>,
    state: AsyncState<T>,
}

// The `Registry` is a standardized resources manager that defines a set of interface for creation,
// destruction, sharing and lifetime management. It is used in all the built-in crayon modules.
pub struct Registry<T> {
    res: Arc<ResourceSystemShared>,
    items: ObjectPool<Entry<T>>,
    redirects: FastHashMap<Uuid, Handle>,
}

impl<T> Registry<T> {
    /// Creates a new and empty `Registry`.
    pub fn new(res: Arc<ResourceSystemShared>) -> Self {
        Registry {
            res: res,
            items: ObjectPool::new(),
            redirects: FastHashMap::default(),
        }
    }

    /// Creates a resource with provided value instance.
    ///
    /// A associated `Handle` is returned.
    pub fn create(&mut self, value: T) -> Handle {
        let entry = Entry {
            rc: 1,
            uuid: None,
            state: AsyncState::Ok(value),
        };

        self.items.create(entry)
    }

    /// Deletes a resource from registery.
    pub fn delete(&mut self, handle: Handle) -> Option<T> {
        let free = self.items
            .get_mut(handle)
            .map(|entry| {
                entry.rc -= 1;
                entry.rc == 0
            })
            .unwrap_or(false);

        if free {
            let entry = self.items.free(handle).unwrap();

            if let Some(uuid) = entry.uuid {
                self.redirects.remove(&uuid);
            }

            if let AsyncState::Ok(value) = entry.state {
                return Some(value);
            }
        }

        None
    }

    /// Gets the stored value.
    pub fn get(&self, handle: Handle) -> Option<&T> {
        self.items.get(handle).and_then(|entry| {
            if let AsyncState::Ok(ref v) = entry.state {
                Some(v)
            } else {
                None
            }
        })
    }

    /// Blocks current thread until loader is finished.
    pub fn promise(&self, handle: Handle) -> Option<Arc<Promise>> {
        self.items.get(handle).and_then(|entry| {
            if let AsyncState::NotReady(ref v) = entry.state {
                Some(v.clone())
            } else {
                None
            }
        })
    }

    /// Creates a resource from readable location.
    pub fn create_from<L: Loader>(&mut self, loader: L, location: Location) -> Result<Handle> {
        let uuid = self.res.redirect(location).ok_or_else(|| {
            format_err!(
                "Undefined virtual filesystem with identifier {}.",
                location.vfs()
            )
        })?;

        self.create_from_uuid(loader, uuid)
    }

    /// Creates a resource from Uuid.
    pub fn create_from_uuid<L: Loader>(&mut self, loader: L, uuid: Uuid) -> Result<Handle> {
        if let Some(handle) = self.redirects.get(&uuid).cloned() {
            self.items.get_mut(handle).unwrap().rc += 1;
            Ok(handle)
        } else {
            let promise = self.res.load(loader, uuid)?;
            let entry = Entry {
                rc: 1,
                uuid: Some(uuid),
                state: AsyncState::NotReady(promise),
            };

            let handle = self.items.create(entry);
            self.redirects.insert(uuid, handle);
            Ok(handle)
        }
    }

    /// Updates a resource asynchronously.
    ///
    /// Notes that the resource might be deleted before updating happens. The value will
    /// be returned if its freed.
    pub fn update(&mut self, handle: Handle, value: T) -> Option<T> {
        if let Some(entry) = self.items.get_mut(handle) {
            entry.state = AsyncState::Ok(value);
            None
        } else {
            Some(value)
        }
    }
}
