use gl;
use glutin;
use glutin::GlContext;

use crate::errors::*;
use crate::math::prelude::Vector2;

use super::super::super::events::Event;
use super::super::super::WindowParams;
use super::super::Visitor;
use super::types;

pub struct GlutinVisitor {
    window: glutin::GlWindow,
    events_loop: glutin::EventsLoop,
}

impl GlutinVisitor {
    pub fn from(params: WindowParams) -> Result<Self> {
        let builder = glutin::WindowBuilder::new()
            .with_title(params.title)
            .with_dimensions(glutin::dpi::LogicalSize::new(
                f64::from(params.size.x),
                f64::from(params.size.y),
            ))
            .with_multitouch();

        let context = glutin::ContextBuilder::new()
            .with_multisampling(params.multisample as u16)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Latest)
            .with_vsync(params.vsync);

        let events_loop = glutin::EventsLoop::new();
        let window = glutin::GlWindow::new(builder, context, &events_loop).unwrap();
        let mut visitor = GlutinVisitor {
            window,
            events_loop,
        };

        let size = visitor.dimensions();
        let dpr = visitor.device_pixel_ratio();
        let dims = Vector2::new((size.x as f32 * dpr) as u32, (size.y as f32 * dpr) as u32);

        visitor.events_loop.poll_events(|_| {});
        visitor.resize(dims);

        unsafe {
            visitor.window.make_current()?;
            gl::load_with(|symbol| visitor.window.get_proc_address(symbol) as *const _);
        }

        Ok(visitor)
    }
}

impl Visitor for GlutinVisitor {
    #[inline]
    fn show(&self) {
        self.window.show();
    }

    #[inline]
    fn hide(&self) {
        self.window.hide();
    }

    #[inline]
    fn position(&self) -> Vector2<i32> {
        let pos = self.window.get_position().unwrap();
        Vector2::new(pos.x as i32, pos.y as i32)
    }

    #[inline]
    fn dimensions(&self) -> Vector2<u32> {
        let size = self.window.get_inner_size().unwrap();
        Vector2::new(size.width as u32, size.height as u32)
    }

    #[inline]
    fn device_pixel_ratio(&self) -> f32 {
        self.window.get_hidpi_factor() as f32
    }

    #[inline]
    fn resize(&self, dimensions: Vector2<u32>) {
        let size = glutin::dpi::PhysicalSize::new(f64::from(dimensions.x), f64::from(dimensions.y));
        self.window.resize(size)
    }

    #[inline]
    fn poll_events(&mut self, events: &mut Vec<Event>) {
        let dims = self.dimensions();
        self.events_loop.poll_events(|v| {
            if let Some(e) = types::from_event(v, dims) {
                events.push(e);
            }
        });
    }

    #[inline]
    fn is_current(&self) -> bool {
        self.window.is_current()
    }

    #[inline]
    fn make_current(&self) -> Result<()> {
        unsafe {
            self.window.make_current()?;
            Ok(())
        }
    }

    #[inline]
    fn swap_buffers(&self) -> Result<()> {
        self.window.swap_buffers()?;
        Ok(())
    }
}
