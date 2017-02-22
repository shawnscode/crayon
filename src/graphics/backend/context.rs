use std::sync::Arc;
use glutin;
use gl;

use super::errors::*;
use super::capabilities::{Capabilities, Version};

pub struct Context {
    window: Arc<glutin::Window>,
    capabilities: Capabilities,
}

impl Context {
    pub fn new(window: Arc<glutin::Window>) -> Result<Self> {
        unsafe {
            window.make_current()?;
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

            let capabilities = Capabilities::parse()?;
            Context::check_minimal_requirements(&capabilities)?;

            let context = Context {
                window: window,
                capabilities: capabilities,
            };

            Ok(context)
        }
    }

    fn check_minimal_requirements(caps: &Capabilities) -> Result<()> {
        if caps.version < Version::GL(1, 5) && caps.version < Version::ES(2, 0) &&
           (!caps.extensions.gl_arb_vertex_buffer_object ||
            !caps.extensions.gl_arb_map_buffer_range) {
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
}

impl Context {
    /// Returns the capabilities of this OpenGL implementation.
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// Returns true if this context is the current one in this thread.
    pub fn is_current(&self) -> bool {
        self.window.is_current()
    }

    /// Set the context as the active context in this thread.
    pub fn make_current(&self) -> Result<()> {
        unsafe { self.window.make_current().chain_err(|| "unable to make context current.") }
    }

    /// Swaps the buffers in case of double or triple buffering.
    ///
    /// **Warning**: if you enabled vsync, this function will block until the
    /// next time the screen is refreshed. However drivers can choose to
    /// override your vsync settings, which means that you can't know in advance
    /// whether swap_buffers will block or not.
    pub fn swap_buffers(&self) -> Result<()> {
        self.window.swap_buffers().chain_err(|| "unable to swap buffers.")
    }
}