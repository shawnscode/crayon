use std::sync::{Arc, Mutex, RwLock};

use crate::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crate::errors::*;
use crate::utils::object_pool::ObjectPool;

use super::backends::{self, Visitor};

impl_handle!(EventListenerHandle);

pub trait EventListener {
    fn on(&mut self, v: &String) -> Result<()>;
}

/// Represents an OpenGL context and the window or environment around it.
pub struct NetworkSystem {
    lis: LifecycleListenerHandle,
    state: Arc<NetworkState>,
}

struct NetworkState {
    visitor: RwLock<Box<dyn Visitor>>,
    events: Mutex<Vec<String>>,
    listeners: Mutex<ObjectPool<EventListenerHandle, Arc<Mutex<dyn EventListener>>>>,
}

impl LifecycleListener for Arc<NetworkState> {
    fn on_pre_update(&mut self) -> crate::errors::Result<()> {
        // Polls events from window, and returns the iterator over them.
        let mut events = self.events.lock().unwrap();
        events.clear();

        let mut visitor = self.visitor.write().unwrap();
        visitor.poll_events(&mut events);

        Ok(())
    }

    fn on_post_update(&mut self) -> crate::errors::Result<()> {
        Ok(())
    }
}
impl EventListener for Arc<NetworkState> {
    fn on(&mut self, _v: &String) -> Result<()> {
        Ok(())
    }
}
impl Drop for NetworkSystem {
    fn drop(&mut self) {
        crate::application::detach(self.lis);
    }
}

#[allow(dead_code)]
impl NetworkSystem {
    pub fn new()-> Self{
        let state = Arc::new(NetworkState {
            listeners: Mutex::new(ObjectPool::new()),
            events: Mutex::new(Vec::new()),
            visitor: RwLock::new(backends::new().unwrap()),
        });

        let window = NetworkSystem {
            state: state.clone(),
            lis: crate::application::attach(state),
        };
        window
    }
    /// Creates a new `NetworkSystem` and initalize OpenGL context.
    pub fn create_connection(&self,param:String) -> Result<()> {
        self.state.visitor.write().unwrap().create_connection(param).unwrap();
        Ok(())
    }
    pub fn receive(&self) ->Vec<String>{
        self.state.events.lock().unwrap().clone()
    }
    pub fn send(&self,p:String){
        self.state.visitor.write().unwrap().send(p);
    }

}
