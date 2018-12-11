use std::sync::{Arc, Mutex, RwLock};

use crate::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crate::errors::*;
use crate::math::prelude::Vector2;
use crate::utils::object_pool::ObjectPool;

use super::backends::{self, Visitor};
use super::events::Event;
use super::WindowParams;

impl_handle!(EventListenerHandle);

pub trait EventListener {
    fn on(&mut self, v: &Event) -> Result<()>;
}

/// Represents an OpenGL context and the window or environment around it.
pub struct WindowSystem {
    lis: LifecycleListenerHandle,
    state: Arc<WindowState>,
}

struct WindowState {
    visitor: RwLock<Box<dyn Visitor>>,
    events: Mutex<Vec<Event>>,
    last_frame_listeners: Mutex<Vec<Arc<Mutex<dyn EventListener>>>>,
    listeners: Mutex<ObjectPool<EventListenerHandle, Arc<Mutex<dyn EventListener>>>>,
}

impl LifecycleListener for Arc<WindowState> {
    fn on_pre_update(&mut self) -> crate::errors::Result<()> {
        // Polls events from window, and returns the iterator over them.
        let mut events = self.events.lock().unwrap();
        events.clear();

        let mut visitor = self.visitor.write().unwrap();
        visitor.poll_events(&mut events);

        let mut last_frame_listeners = self.last_frame_listeners.lock().unwrap();

        {
            let listeners = self.listeners.lock().unwrap();
            last_frame_listeners.clear();
            last_frame_listeners.extend(listeners.values().cloned());
        }

        for lis in last_frame_listeners.iter() {
            let mut lis = lis.lock().unwrap();
            for v in events.iter() {
                lis.on(v)?;
            }
        }

        Ok(())
    }

    fn on_post_update(&mut self) -> crate::errors::Result<()> {
        // Swaps the buffers in case of double or triple buffering.
        //
        // **Warning**: if you enabled vsync, this function will block until the next time the screen
        // is refreshed. However drivers can choose to override your vsync settings, which means that
        // you can't know in advance whether swap_buffers will block or not.
        self.visitor.read().unwrap().swap_buffers()?;
        Ok(())
    }
}

impl Drop for WindowSystem {
    fn drop(&mut self) {
        crate::application::detach(self.lis);
    }
}

impl WindowSystem {
    /// Creates a new `WindowSystem` and initalize OpenGL context.
    pub fn from(params: WindowParams) -> Result<Self> {
        let state = Arc::new(WindowState {
            last_frame_listeners: Mutex::new(Vec::new()),
            listeners: Mutex::new(ObjectPool::new()),
            events: Mutex::new(Vec::new()),
            visitor: RwLock::new(backends::new(params)?),
        });

        let window = WindowSystem {
            state: state.clone(),
            lis: crate::application::attach(state),
        };

        Ok(window)
    }

    /// Creates a new `Window` with headless context.
    pub fn headless() -> Self {
        let state = Arc::new(WindowState {
            last_frame_listeners: Mutex::new(Vec::new()),
            listeners: Mutex::new(ObjectPool::new()),
            events: Mutex::new(Vec::new()),
            visitor: RwLock::new(backends::new_headless()),
        });

        WindowSystem {
            state: state.clone(),
            lis: crate::application::attach(state),
        }
    }

    /// Adds a event listener.
    pub fn add_event_listener<T: EventListener + 'static>(&self, lis: T) -> EventListenerHandle {
        let lis = Arc::new(Mutex::new(lis));
        self.state.listeners.lock().unwrap().create(lis)
    }

    /// Removes a event listener from window.
    pub fn remove_event_listener(&self, handle: EventListenerHandle) {
        self.state.listeners.lock().unwrap().free(handle);
    }

    /// Shows the window if it was hidden.
    ///
    /// # Platform-specific
    ///
    /// Has no effect on mobile platform.
    #[inline]
    pub fn show(&self) {
        self.state.visitor.read().unwrap().show();
    }

    /// Hides the window if it was visible.
    ///
    /// # Platform-specific
    ///
    /// Has no effect on mobile platform.
    #[inline]
    pub fn hide(&self) {
        self.state.visitor.read().unwrap().hide();
    }

    /// Set the context as the active context in this thread.
    #[inline]
    pub fn make_current(&self) -> Result<()> {
        self.state.visitor.read().unwrap().make_current()
    }

    /// Returns true if this context is the current one in this thread.
    #[inline]
    pub fn is_current(&self) -> bool {
        self.state.visitor.read().unwrap().is_current()
    }

    /// Returns the position of the lower-left hand corner of the window relative to the lower-left
    /// hand corner of the desktop. Note that the lower-left hand corner of the desktop is not
    /// necessarily the same as the screen. If the user uses a desktop with multiple monitors,
    /// the lower-left hand corner of the desktop is the lower-left hand corner of the monitor at
    /// the lower-left of the desktop.
    ///
    /// The coordinates can be negative if the lower-left hand corner of the window is outside of
    /// the visible screen region.
    #[inline]
    pub fn position(&self) -> Vector2<i32> {
        self.state.visitor.read().unwrap().position()
    }

    /// Returns the size in *points* of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These are
    /// the size of the frame buffer.
    #[inline]
    pub fn dimensions(&self) -> Vector2<u32> {
        self.state.visitor.read().unwrap().dimensions()
    }

    /// Returns the ratio between the backing framebuffer resolution and the window size in
    /// screen pixels. This is typically one for a normal display and two for a retina display.
    #[inline]
    pub fn device_pixel_ratio(&self) -> f32 {
        self.state.visitor.read().unwrap().device_pixel_ratio()
    }

    /// Resize the GL context.
    #[inline]
    pub fn resize(&self, dimensions: Vector2<u32>) {
        self.state.visitor.read().unwrap().resize(dimensions);
    }
}
