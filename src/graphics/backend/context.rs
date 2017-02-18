use std::sync::Arc;
use glutin;
use gl;

use super::*;

pub struct GlutinContext {
    window: Arc<glutin::Window>,
}

impl GlutinContext {
    pub fn new(window: Arc<glutin::Window>) -> Result<Self> {
        unsafe {
            window.make_current()?;
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        }

        Ok(GlutinContext { window: window })
    }
}

impl Context for GlutinContext {
    fn swap_buffers(&self) -> Result<()> {
        self.window.swap_buffers().chain_err(|| "unable to swap buffers.")
    }

    fn is_current(&self) -> bool {
        self.window.is_current()
    }

    unsafe fn make_current(&self) -> Result<()> {
        self.window.make_current().chain_err(|| "unable to make context current.")
    }
}