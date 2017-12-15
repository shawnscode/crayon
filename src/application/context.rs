//! The context of systems that could be accessed from multi-thread environments.
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::any::{Any, TypeId};

pub trait ContextSystem {
    type Shared: Send + Sync + 'static;
}

/// The context of sub-systems that could be accessed from multi-thread environments.
pub struct Context {
    shareds: HashMap<TypeId, Box<Any + Send + Sync>>,
    shutdown: RwLock<bool>,
}

impl Context {
    /// Gets multi-thread friendly APIs of specified systems.
    pub fn shared<T>(&self) -> &Arc<T::Shared>
        where T: ContextSystem + 'static
    {
        let tid = TypeId::of::<T>();
        Self::cast::<T>(self.shareds.get(&tid).unwrap().as_ref())
    }

    /// Shutdown the whole application.
    pub fn shutdown(&self) {
        *self.shutdown.write().unwrap() = true;
    }

    #[inline]
    fn cast<T>(any: &Any) -> &Arc<T::Shared>
        where T: ContextSystem + 'static
    {
        any.downcast_ref::<Arc<T::Shared>>().unwrap()
    }

    pub(crate) fn new() -> Self {
        Context {
            shareds: HashMap::new(),
            shutdown: RwLock::new(false),
        }
    }

    pub(crate) fn insert<T>(&mut self, v: Arc<T::Shared>)
        where T: ContextSystem + 'static
    {
        let tid = TypeId::of::<T>();
        self.shareds.insert(tid, Box::new(v));
    }

    pub(crate) fn is_shutdown(&self) -> bool {
        *self.shutdown.read().unwrap()
    }
}