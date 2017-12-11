//! An OpenGL context and the environment around it.

use std::default::Default;
use std::sync::Arc;

use gl;

use glutin;
use glutin::GlContext;

use input;
use super::backend::capabilities::{Capabilities, Version};
use super::errors::*;

/// Represents an OpenGL context and the Window or environment around it, its just
/// simple wrappers to [glutin](https://github.com/tomaka/glutin) right now.
pub struct Window {
    window: Arc<glutin::GlWindow>,
    capabilities: Capabilities,
}

impl Window {
    /// Creates a builder to initilize OpenGL context and a window for platforms
    /// where this is appropriate.
    pub fn build() -> WindowBuilder {
        WindowBuilder::new()
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

    /// Returns the size in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    /// These are the dimensions of the frame buffer.
    #[inline]
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.window.get_inner_size_pixels()
    }

    /// Returns the size in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    #[inline]
    pub fn point_dimensions(&self) -> Option<(u32, u32)> {
        self.window.get_inner_size_points()
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

    /// Returns the capabilities of this OpenGL implementation.
    #[inline]
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
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

    pub fn build(self, events: &input::InputSystem) -> Result<Window> {
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

        let window = glutin::WindowBuilder::new()
            .with_title(self.title.clone())
            .with_dimensions(self.size.0, self.size.1)
            .with_multitouch();

        let context = glutin::ContextBuilder::new()
            .with_multisampling(self.multisample)
            .with_gl_profile(profile)
            .with_gl(api)
            .with_vsync(self.vsync);

        let window = glutin::GlWindow::new(window, context, &events.underlaying())?;

        let capabilities = unsafe {
            window.make_current()?;
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            Capabilities::parse()?
        };

        println!("{:#?}", capabilities);
        check_minimal_requirements(&capabilities)?;
        Ok(Window {
               window: Arc::new(window),
               capabilities: capabilities,
           })
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
            title: "Window".to_owned(),
            position: (0, 0),
            size: (512, 512),
            vsync: false,
            multisample: 0,
            api: OpenGLAPI::Lastest,
            profile: OpenGLProfile::Core,
        }
    }
}

fn check_minimal_requirements(caps: &Capabilities) -> Result<()> {
    if caps.version < Version::GL(1, 5) && caps.version < Version::ES(2, 0) &&
       (!caps.extensions.gl_arb_vertex_buffer_object || !caps.extensions.gl_arb_map_buffer_range) {
        bail!("OpenGL implementation doesn't support vertex buffer objects.");
    }

    if caps.version < Version::GL(2, 0) && caps.version < Version::ES(2, 0) &&
       (!caps.extensions.gl_arb_shader_objects || !caps.extensions.gl_arb_vertex_shader ||
        !caps.extensions.gl_arb_fragment_shader) {
        bail!("OpenGL implementation doesn't support vertex/fragment shaders.");
    }

    if caps.version < Version::GL(3, 0) && caps.version < Version::ES(2, 0) &&
       !caps.extensions.gl_ext_framebuffer_object &&
       !caps.extensions.gl_arb_framebuffer_object {
        bail!("OpenGL implementation doesn't support framebuffers.");
    }

    if caps.version < Version::ES(2, 0) && caps.version < Version::GL(3, 0) &&
       !caps.extensions.gl_ext_framebuffer_blit {
        bail!("OpenGL implementation doesn't support blitting framebuffers.");
    }

    if caps.version < Version::GL(3, 1) && caps.version < Version::ES(3, 0) &&
       !caps.extensions.gl_arb_uniform_buffer_object {
        bail!("OpenGL implementation doesn't support uniform buffer object.");
    }

    if caps.version < Version::GL(3, 0) && caps.version < Version::ES(3, 0) &&
       !caps.extensions.gl_arb_vertex_array_object &&
       !caps.extensions.gl_apple_vertex_array_object &&
       !caps.extensions.gl_oes_vertex_array_object {
        bail!("OpenGL implementation doesn't support vertex array object.");
    }

    Ok(())
}