use std::default::Default;
use std::sync::Arc;
use gl;
use glutin;

use super::errors::*;

/// The status of application.
#[derive(Debug)]
pub enum ApplicationEvent {
    /// The window has been woken up by another thread.
    Awakened,
    /// The window has been resumed.
    Resumed,
    /// The window has been suspended.
    Suspended,
    /// The window has been closed.
    Closed,
}

/// Window related events.
#[derive(Debug)]
pub enum WindowEvent {
    /// The size of window has changed.
    Resized(u32, u32),
    /// The position of window has changed.
    Moved(u32, u32),
}

pub type KeyboardButton = glutin::VirtualKeyCode;
pub type MouseButton = glutin::MouseButton;

/// Input device event, supports mouse and keyboard only.
#[derive(Debug)]
pub enum InputDeviceEvent {
    /// The window gained focus of user input.
    GainFocus,
    /// The window lost focus of user input.
    LostFocus,

    /// The cursor has moved on the window.
    /// The parameter are the (x, y) coords in pixels relative to the top-left
    /// corner of th window.
    MouseMoved(i32, i32),
    /// Pressed event on mouse has been received.
    MousePressed(MouseButton),
    /// Released event from mouse has been received.
    MouseReleased(MouseButton),

    /// Pressed event on keyboard has been received.
    KeyboardPressed(KeyboardButton),
    /// Released event from keyboard has been received.
    KeyboardReleased(KeyboardButton),
}

#[derive(Debug)]
pub enum Event {
    Application(ApplicationEvent),
    Window(WindowEvent),
    InputDevice(InputDeviceEvent),
}

/// Represents an OpenGL context and the Window or environment around it, its just
/// simple wrappers to [glutin](https://github.com/tomaka/glutin) right now.
pub struct Window {
    window: Arc<glutin::Window>,
}

impl Window {
    /// Creates a builder to initilize OpenGL context and a window for platforms
    /// where this is appropriate.
    pub fn build() -> WindowBuilder {
        WindowBuilder::new()
    }

    pub fn underlaying(&self) -> Arc<glutin::Window> {
        self.window.clone()
    }

    /// Shows the window if it was hidden.
    ///
    /// # Platform-specific
    ///
    /// Has no effect on mobile platform.
    #[inline]
    pub fn show(&self) {
        self.window.show();
    }

    /// Hides the window if it was visible.
    ///
    /// # Platform-specific
    ///
    /// Has no effect on mobile platform.
    #[inline]
    pub fn hide(&self) {
        self.window.hide();
    }

    /// Modifies the title of window.
    #[inline]
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Returns the position of the top-left hand corner of the window relative
    /// to the top-left hand corner of the desktop. Note that the top-left hand
    /// corner of the desktop is not necessarily the same as the screen. If the
    /// user uses a desktop with multiple monitors, the top-left hand corner of
    /// the desktop is the top-left hand corner of the monitor at the top-left
    /// of the desktop.
    /// The coordinates can be negative if the top-left hand corner of the window
    /// is outside of the visible screen region.
    /// Returns None if the window no longer exists.
    #[inline]
    pub fn get_position(&self) -> Option<(i32, i32)> {
        self.window.get_position()
    }

    /// Modifies the position of the window.
    #[inline]
    pub fn set_position(&self, x: i32, y: i32) {
        self.window.set_position(x, y);
    }

    /// Set the context as the active context in this thread.
    #[inline]
    pub fn make_current(&self) -> Result<()> {
        unsafe {
            self.window.make_current()?;
            Ok(())
        }
    }

    /// Returns true if this context is the current one in this thread.
    #[inline]
    pub fn is_current(&self) -> bool {
        self.window.is_current()
    }

    /// Swaps the buffers in case of double or triple buffering.
    ///
    /// **Warning**: if you enabled vsync, this function will block until the
    /// next time the screen is refreshed. However drivers can choose to
    /// override your vsync settings, which means that you can't know in advance
    /// whether swap_buffers will block or not.
    #[inline]
    pub fn swap_buffers(&self) -> Result<()> {
        self.window.swap_buffers()?;
        Ok(())
    }

    /// Returns an iterator that poll for the next event in the window's
    /// events queue. Returns `None` if there is no event in the queue.
    #[inline]
    pub fn poll_events<'a>(&'a self) -> EventIterator<'a> {
        EventIterator { iterator: self.window.poll_events() }
    }
}

// impl From<glutin::ContextError> for Error {
//     fn from(err: glutin::ContextError) -> Error {
//         match err {
//             glutin::ContextError::ContextLost => Error::ContextLost,
//             glutin::ContextError::IoError(v) => Error::IoError(v),
//         }
//     }
// }

/// An iterator for the `poll_events` function.
pub struct EventIterator<'a> {
    iterator: glutin::PollEventsIterator<'a>,
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        while let Some(event) = self.iterator.next() {
            let window_event = match event {
                glutin::Event::Awakened => Event::Application(ApplicationEvent::Awakened),
                glutin::Event::Suspended(v) => {
                    Event::Application(if v {
                        ApplicationEvent::Suspended
                    } else {
                        ApplicationEvent::Resumed
                    })
                }
                glutin::Event::Closed => Event::Application(ApplicationEvent::Closed),

                glutin::Event::Focused(v) => {
                    Event::InputDevice(if v {
                        InputDeviceEvent::GainFocus
                    } else {
                        InputDeviceEvent::LostFocus
                    })
                }
                glutin::Event::MouseMoved(x, y) => {
                    Event::InputDevice(InputDeviceEvent::MouseMoved(x, y))
                }
                glutin::Event::MouseInput(glutin::ElementState::Pressed, button) => {
                    Event::InputDevice(InputDeviceEvent::MousePressed(button))
                }
                glutin::Event::MouseInput(glutin::ElementState::Released, button) => {
                    Event::InputDevice(InputDeviceEvent::MouseReleased(button))
                }
                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(vkey)) => {
                    Event::InputDevice(InputDeviceEvent::KeyboardPressed(vkey))
                }
                glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(vkey)) => {
                    Event::InputDevice(InputDeviceEvent::KeyboardReleased(vkey))
                }
                _ => continue,
            };

            return Some(window_event);
        }

        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

/// Describes the requested OpenGL context profiles.
pub enum OpenGLProfile {
    Compatibility,
    Core,
}

/// Describe the requested OpenGL api.
pub enum OpenGLAPI {
    Lastest,
    GL(u8, u8),
    GLES(u8, u8),
}

/// Struct that allow you to build window.
pub struct WindowBuilder {
    title: String,
    position: (i32, i32),
    size: (u32, u32),
    vsync: bool,
    multisample: u16,
    api: OpenGLAPI,
    profile: OpenGLProfile,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        Default::default()
    }

    pub fn build(self) -> Result<Window> {
        let profile = match self.profile {
            OpenGLProfile::Core => glutin::GlProfile::Core,
            OpenGLProfile::Compatibility => glutin::GlProfile::Compatibility,
        };

        let api = match self.api {
            OpenGLAPI::Lastest => glutin::GlRequest::Latest,
            OpenGLAPI::GL(major, minor) => {
                glutin::GlRequest::Specific(glutin::Api::OpenGl, (major, minor))
            }
            OpenGLAPI::GLES(major, minor) => {
                glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (major, minor))
            }
        };

        let mut builder = glutin::WindowBuilder::new()
            .with_title(self.title.clone())
            .with_dimensions(self.size.0, self.size.1)
            .with_multisampling(self.multisample)
            .with_multitouch()
            .with_gl_profile(profile)
            .with_gl(api);

        if self.vsync {
            builder = builder.with_vsync();
        }

        let window = builder.build()?;

        unsafe {
            window.make_current()?;
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        }

        Ok(Window { window: Arc::new(window) })
    }

    /// Requests a specific title for the window.
    #[inline]
    pub fn with_title<T: Into<String>>(&mut self, title: T) -> &mut Self {
        self.title = title.into();
        self
    }

    /// Requests a specific position for window.
    #[inline]
    pub fn with_position(&mut self, position: (i32, i32)) -> &mut Self {
        self.position = position;
        self
    }

    /// Requests the window to be of specific dimensions.
    #[inline]
    pub fn with_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        self.size = (width, height);
        self
    }

    /// Sets the multisampling level to request. A value of 0 indicates that
    /// multisampling must not be enabled.
    #[inline]
    pub fn with_multisample(&mut self, multisample: u16) -> &mut Self {
        self.multisample = multisample;
        self
    }

    /// Sets the desired OpenGL context profile.
    #[inline]
    pub fn with_profile(&mut self, profile: OpenGLProfile) -> &mut Self {
        self.profile = profile;
        self
    }

    /// Sets how the backend should choose the OpenGL API and version.
    #[inline]
    pub fn with_api(&mut self, api: OpenGLAPI) -> &mut Self {
        self.api = api;
        self
    }
}

impl Default for WindowBuilder {
    fn default() -> WindowBuilder {
        WindowBuilder {
            title: "Lemon3D - Window".to_owned(),
            position: (0, 0),
            size: (512, 512),
            vsync: false,
            multisample: 0,
            api: OpenGLAPI::Lastest,
            profile: OpenGLProfile::Core,
        }
    }
}
