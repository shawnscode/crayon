use std::sync::{Arc, Mutex};

use crate::utils::object_pool::ObjectPool;

impl_handle!(LifecycleListenerHandle);

pub trait LifecycleListener {
    fn on_pre_update(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn on_update(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn on_render(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn on_post_update(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    fn on_exit(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }
}

pub struct LifecycleSystem {
    last_frame_lifecycles: Mutex<Vec<Arc<Mutex<LifecycleListener>>>>,
    lifecycles: Mutex<ObjectPool<LifecycleListenerHandle, Arc<Mutex<LifecycleListener>>>>,
}

impl LifecycleSystem {
    pub fn new() -> Self {
        LifecycleSystem {
            last_frame_lifecycles: Mutex::new(Vec::new()),
            lifecycles: Mutex::new(ObjectPool::new()),
        }
    }

    #[inline]
    pub fn attach<T>(&self, lis: T) -> LifecycleListenerHandle
    where
        T: LifecycleListener + 'static,
    {
        self.lifecycles
            .lock()
            .unwrap()
            .create(Arc::new(Mutex::new(lis)))
    }

    #[inline]
    pub fn detach(&self, handle: LifecycleListenerHandle) {
        // Makes sure that the lock has been freed before the drop of
        // LifecycleListener, since it might cause dead lock.
        #[allow(clippy::let_and_return)]
        let _ = {
            let v = self.lifecycles.lock().unwrap().free(handle);
            v
        };
    }

    #[inline]
    pub fn foreach<T>(&self, func: T) -> Result<(), failure::Error>
    where
        T: Fn(&mut dyn LifecycleListener) -> Result<(), failure::Error>,
    {
        let mut last_frame_lifecycles = self.last_frame_lifecycles.lock().unwrap();

        {
            let lifecycles = self.lifecycles.lock().unwrap();
            last_frame_lifecycles.extend(lifecycles.values().cloned());
        }

        for v in last_frame_lifecycles.drain(..) {
            func(&mut *v.lock().unwrap())?;
        }

        Ok(())
    }

    #[inline]
    pub fn foreach_rev<T>(&self, func: T) -> Result<(), failure::Error>
    where
        T: Fn(&mut dyn LifecycleListener) -> Result<(), failure::Error>,
    {
        let mut last_frame_lifecycles = self.last_frame_lifecycles.lock().unwrap();

        {
            let lifecycles = self.lifecycles.lock().unwrap();
            last_frame_lifecycles.extend(lifecycles.values().rev().cloned());
        }

        for v in last_frame_lifecycles.drain(..) {
            func(&mut *v.lock().unwrap())?;
        }

        Ok(())
    }
}
