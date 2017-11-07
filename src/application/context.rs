use std::collections::HashMap;
use std::sync::Arc;
use std::any::{Any, TypeId};

pub trait ContextSystem {
    type Shared: Send + Sync + 'static;
}

pub struct Context {
    shareds: HashMap<TypeId, Box<Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Self {
        Context { shareds: HashMap::new() }
    }

    pub fn insert<T>(&mut self, v: Arc<T::Shared>)
        where T: ContextSystem + 'static
    {
        let tid = TypeId::of::<T>();
        self.shareds.insert(tid, Box::new(v));
    }

    pub fn shared<T>(&self) -> &Arc<T::Shared>
        where T: ContextSystem + 'static
    {
        let tid = TypeId::of::<T>();
        Self::cast::<T>(self.shareds.get(&tid).unwrap().as_ref())
    }

    #[inline]
    fn cast<T>(any: &Any) -> &Arc<T::Shared>
        where T: ContextSystem + 'static
    {
        any.downcast_ref::<Arc<T::Shared>>().unwrap()
    }
}