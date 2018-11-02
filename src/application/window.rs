use std::slice::Iter;
use std::sync::{Arc, RwLock};

use errors::*;
use math::prelude::Vector2;

use super::backends::{self, Visitor};
use super::events::Event;
use super::settings::WindowParams;

/// Represents an OpenGL context and the window or environment around it.
pub struct Window {
    visitor: Box<Visitor>,
    events: Vec<Event>,
    shared: Arc<WindowShared>,
}

impl Window {
    /// Creates a new `Window` and initalize OpenGL context.
    pub fn new(params: WindowParams) -> Result<Self> {
        let mut window = Window {
            visitor: backends::new(params)?,
            events: Vec::new(),
            shared: Arc::new(WindowShared {
                dimensions: RwLock::new(Vector2::new(0, 0)),
                dimensions_in_points: RwLock::new(Vector2::new(0, 0)),
                hidpi: RwLock::new(1.0),
            }),
        };

        window.advance();
        Ok(window)
    }

    /// Creates a new `Window` with headless context.
    pub fn headless() -> Self {
        Window {
            visitor: backends::new_headless(),
            events: Vec::new(),
            shared: Arc::new(WindowShared {
                dimensions: RwLock::new(Vector2::new(0, 0)),
                dimensions_in_points: RwLock::new(Vector2::new(0, 0)),
                hidpi: RwLock::new(1.0),
            }),
        }
    }

    /// Gets the multi-thread friendly parts of `Window`.
    pub fn shared(&self) -> Arc<WindowShared> {
        self.shared.clone()
    }

    /// Shows the window if it was hidden.
    ///
    /// # Platform-specific
    ///
    /// Has no effect on mobile platform.
    #[inline]
    pub fn show(&self) {
        self.visitor.show();
    }

    /// Hides the window if it was visible.
    ///
    /// # Platform-specific
    ///
    /// Has no effect on mobile platform.
    #[inline]
    pub fn hide(&self) {
        self.visitor.hide();
    }

    /// Set the context as the active context in this thread.
    #[inline]
    pub fn make_current(&self) -> Result<()> {
        self.visitor.make_current()
    }

    /// Returns true if this context is the current one in this thread.
    #[inline]
    pub fn is_current(&self) -> bool {
        self.visitor.is_current()
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
    pub fn position_in_points(&self) -> Vector2<i32> {
        self.visitor.position_in_points()
    }

    /// Returns the size in *points* of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These are
    /// the size of the frame buffer.
    #[inline]
    pub fn dimensions_in_points(&self) -> Vector2<u32> {
        self.visitor.dimensions_in_points()
    }

    /// Returns the size in *pixels* of the client area of the window.
    #[inline]
    pub fn dimensions(&self) -> Vector2<u32> {
        let size = self.dimensions_in_points();
        let hi = self.hidpi();
        Vector2::new((size.x as f32 * hi) as u32, (size.y as f32 * hi) as u32)
    }

    /// Returns the ratio between the backing framebuffer resolution and the window size in
    /// screen pixels. This is typically one for a normal display and two for a retina display.
    #[inline]
    pub fn hidpi(&self) -> f32 {
        self.visitor.hidpi()
    }

    /// Swaps the buffers in case of double or triple buffering.
    ///
    /// **Warning**: if you enabled vsync, this function will block until the next time the screen
    /// is refreshed. However drivers can choose to override your vsync settings, which means that
    /// you can't know in advance whether swap_buffers will block or not.
    #[inline]
    pub(crate) fn swap_buffers(&self) -> Result<()> {
        self.visitor.swap_buffers()?;
        Ok(())
    }

    /// Resize the GL context.
    #[inline]
    pub(crate) fn resize(&self, dimensions: Vector2<u32>) {
        self.visitor.resize(dimensions);
    }

    /// Polls events from window, and returns the iterator over them.
    pub(crate) fn advance(&mut self) -> Iter<Event> {
        *self.shared.dimensions_in_points.write().unwrap() = self.dimensions_in_points();
        *self.shared.dimensions.write().unwrap() = self.dimensions();
        *self.shared.hidpi.write().unwrap() = self.hidpi();

        self.events.clear();
        self.visitor.poll_events(&mut self.events);
        self.events.iter()
    }
}

pub struct WindowShared {
    dimensions_in_points: RwLock<Vector2<u32>>,
    dimensions: RwLock<Vector2<u32>>,
    hidpi: RwLock<f32>,
}

impl WindowShared {
    /// Returns the size in *points* of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These are
    /// the size of the frame buffer.
    #[inline]
    pub fn dimensions_in_points(&self) -> Vector2<u32> {
        *self.dimensions_in_points.read().unwrap()
    }

    /// Returns the size in *pixels* of the client area of the window.
    #[inline]
    pub fn dimensions(&self) -> Vector2<u32> {
        *self.dimensions.read().unwrap()
    }

    /// Returns the ratio between the backing framebuffer resolution and the window size in
    /// screen pixels. This is typically one for a normal display and two for a retina display.
    #[inline]
    pub fn hidpi(&self) -> f32 {
        *self.hidpi.read().unwrap()
    }
}
