//! An OpenGL context and the environment around it.

use std::slice::Iter;
use std::sync::{Arc, RwLock};

use glutin;
use glutin::GlContext;

use math;

use super::event::*;
use super::settings::WindowParams;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "[GLUTIN] {}", _0)]
    Context(String),
    #[fail(display = "[GLUTIN] {}", _0)]
    Creation(String),
}

impl From<glutin::CreationError> for Error {
    fn from(err: glutin::CreationError) -> Error {
        Error::Creation(format!("{}", err))
    }
}

impl From<glutin::ContextError> for Error {
    fn from(err: glutin::ContextError) -> Error {
        Error::Context(format!("{}", err))
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// Represents an OpenGL context and the window or environment around it, its just
/// simple wrappers to [glutin](https://github.com/tomaka/glutin) right now.
pub struct Window {
    events_loop: Option<glutin::EventsLoop>,
    visitor: Box<Visitor>,
    events: Vec<Event>,
    shared: Arc<WindowShared>,
}

impl Window {
    /// Creates a new `Window` and initalize OpenGL context.
    pub fn new(params: WindowParams) -> Result<Self> {
        let builder = glutin::WindowBuilder::new()
            .with_title(params.title)
            .with_dimensions(glutin::dpi::LogicalSize::new(
                params.size.x as f64,
                params.size.y as f64,
            ))
            .with_multitouch();

        let context = glutin::ContextBuilder::new()
            .with_multisampling(params.multisample as u16)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Latest)
            .with_vsync(params.vsync);

        let events_loop = glutin::EventsLoop::new();
        let device = glutin::GlWindow::new(builder, context, &events_loop)?;
        unsafe {
            device.make_current()?;
        }

        let window = Window {
            visitor: Box::new(GlutinVisitor(device)),
            events_loop: Some(events_loop),
            events: Vec::new(),
            shared: Arc::new(WindowShared {
                dimensions: RwLock::new(math::Vector2::new(0, 0)),
                dimensions_in_points: RwLock::new(math::Vector2::new(0, 0)),
                hidpi: RwLock::new(1.0),
            }),
        };

        Ok(window)
    }

    /// Creates a new `Window` with headless context.
    pub fn headless() -> Self {
        Window {
            visitor: Box::new(HeadlessVisitor {}),
            events_loop: None,
            events: Vec::new(),
            shared: Arc::new(WindowShared {
                dimensions: RwLock::new(math::Vector2::new(0, 0)),
                dimensions_in_points: RwLock::new(math::Vector2::new(0, 0)),
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

    /// Polls events from window, and returns the iterator over them.
    pub fn advance(&mut self) -> Iter<Event> {
        *self.shared.dimensions_in_points.write().unwrap() = self.dimensions_in_points();
        *self.shared.dimensions.write().unwrap() = self.dimensions();
        *self.shared.hidpi.write().unwrap() = self.hidpi();

        self.events.clear();

        {
            let dims = self.dimensions_in_points();
            let events = &mut self.events;

            if let Some(ref mut events_loop) = self.events_loop {
                events_loop.poll_events(|evt| {
                    if let Some(v) = from_event(evt, dims) {
                        events.push(v);
                    }
                });
            }
        }

        self.events.iter()
    }

    /// Swaps the buffers in case of double or triple buffering.
    ///
    /// **Warning**: if you enabled vsync, this function will block until the next time the screen
    /// is refreshed. However drivers can choose to override your vsync settings, which means that
    /// you can't know in advance whether swap_buffers will block or not.
    #[inline]
    pub fn swap_buffers(&self) -> Result<()> {
        self.visitor.swap_buffers()?;
        Ok(())
    }

    /// Resize the GL context.
    #[inline]
    pub fn resize(&self, dimensions: math::Vector2<u32>) {
        self.visitor.resize(dimensions);
    }

    /// Returns the address of an OpenGL function.
    ///
    /// Contrary to wglGetProcAddress, all available OpenGL functions return an address.
    #[inline]
    pub fn get_proc_address(&self, addr: &str) -> *const () {
        self.visitor.get_proc_address(addr)
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
    pub fn position_in_points(&self) -> math::Vector2<i32> {
        self.visitor.position()
    }

    /// Returns the size in *points* of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These are
    /// the size of the frame buffer.
    #[inline]
    pub fn dimensions_in_points(&self) -> math::Vector2<u32> {
        self.visitor.dimensions()
    }

    /// Returns the size in *pixels* of the client area of the window.
    #[inline]
    pub fn dimensions(&self) -> math::Vector2<u32> {
        let size = self.dimensions_in_points();
        let hi = self.hidpi();
        math::Vector2::new((size.x as f32 * hi) as u32, (size.y as f32 * hi) as u32)
    }

    /// Returns the ratio between the backing framebuffer resolution and the window size in
    /// screen pixels. This is typically one for a normal display and two for a retina display.
    #[inline]
    pub fn hidpi(&self) -> f32 {
        self.visitor.hidpi()
    }
}

pub struct WindowShared {
    dimensions_in_points: RwLock<math::Vector2<u32>>,
    dimensions: RwLock<math::Vector2<u32>>,
    hidpi: RwLock<f32>,
}

impl WindowShared {
    /// Returns the size in *points* of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These are
    /// the size of the frame buffer.
    #[inline]
    pub fn dimensions_in_points(&self) -> math::Vector2<u32> {
        *self.dimensions_in_points.read().unwrap()
    }

    /// Returns the size in *pixels* of the client area of the window.
    #[inline]
    pub fn dimensions(&self) -> math::Vector2<u32> {
        *self.dimensions.read().unwrap()
    }

    /// Returns the ratio between the backing framebuffer resolution and the window size in
    /// screen pixels. This is typically one for a normal display and two for a retina display.
    #[inline]
    pub fn hidpi(&self) -> f32 {
        *self.hidpi.read().unwrap()
    }
}

pub trait Visitor {
    fn show(&self);
    fn hide(&self);
    fn position(&self) -> math::Vector2<i32>;
    fn dimensions(&self) -> math::Vector2<u32>;
    fn hidpi(&self) -> f32;
    fn resize(&self, dimensions: math::Vector2<u32>);

    fn is_current(&self) -> bool;
    fn make_current(&self) -> Result<()>;
    fn swap_buffers(&self) -> Result<()>;
    fn get_proc_address(&self, addr: &str) -> *const ();
}

pub struct GlutinVisitor(glutin::GlWindow);

impl Visitor for GlutinVisitor {
    #[inline]
    fn show(&self) {
        self.0.show();
    }

    #[inline]
    fn hide(&self) {
        self.0.hide();
    }

    #[inline]
    fn position(&self) -> math::Vector2<i32> {
        let pos = self.0.get_position().unwrap();
        math::Vector2::new(pos.x as i32, pos.y as i32)
    }

    #[inline]
    fn dimensions(&self) -> math::Vector2<u32> {
        let size = self.0.get_inner_size().unwrap();
        math::Vector2::new(size.width as u32, size.height as u32)
    }

    #[inline]
    fn hidpi(&self) -> f32 {
        self.0.get_hidpi_factor() as f32
    }

    #[inline]
    fn resize(&self, dimensions: math::Vector2<u32>) {
        let size = glutin::dpi::PhysicalSize::new(dimensions.x as f64, dimensions.y as f64);
        self.0.resize(size)
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.0.is_current()
    }

    #[inline]
    fn make_current(&self) -> Result<()> {
        unsafe {
            self.0.make_current()?;
            Ok(())
        }
    }

    #[inline]
    fn swap_buffers(&self) -> Result<()> {
        self.0.swap_buffers()?;
        Ok(())
    }

    #[inline]
    fn get_proc_address(&self, addr: &str) -> *const () {
        self.0.get_proc_address(addr)
    }
}

struct HeadlessVisitor {}

impl Visitor for HeadlessVisitor {
    #[inline]
    fn show(&self) {}

    #[inline]
    fn hide(&self) {}

    #[inline]
    fn position(&self) -> math::Vector2<i32> {
        (0, 0).into()
    }

    #[inline]
    fn dimensions(&self) -> math::Vector2<u32> {
        (0, 0).into()
    }

    #[inline]
    fn hidpi(&self) -> f32 {
        1.0
    }

    #[inline]
    fn resize(&self, _: math::Vector2<u32>) {}

    #[inline]
    fn is_current(&self) -> bool {
        true
    }

    #[inline]
    fn make_current(&self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn swap_buffers(&self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn get_proc_address(&self, _: &str) -> *const () {
        ::std::ptr::null()
    }
}
