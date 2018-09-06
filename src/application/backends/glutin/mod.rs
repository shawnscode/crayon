mod types;

use gl;
use glutin;
use glutin::GlContext;

use application::events::Event;
use errors::*;
use math;

use super::super::settings::WindowParams;
use super::Visitor;

pub struct GlutinVisitor {
    window: glutin::GlWindow,
    events_loop: glutin::EventsLoop,
}

impl GlutinVisitor {
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
        let device = glutin::GlWindow::new(builder, context, &events_loop).unwrap();

        unsafe {
            device.make_current()?;
            gl::load_with(|symbol| device.get_proc_address(symbol) as *const _);
        }

        Ok(GlutinVisitor {
            window: device,
            events_loop: events_loop,
        })
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
    fn position_in_points(&self) -> math::Vector2<i32> {
        let pos = self.window.get_position().unwrap();
        math::Vector2::new(pos.x as i32, pos.y as i32)
    }

    #[inline]
    fn dimensions_in_points(&self) -> math::Vector2<u32> {
        let size = self.window.get_inner_size().unwrap();
        math::Vector2::new(size.width as u32, size.height as u32)
    }

    #[inline]
    fn hidpi(&self) -> f32 {
        self.window.get_hidpi_factor() as f32
    }

    #[inline]
    fn resize(&self, dimensions: math::Vector2<u32>) {
        let size = glutin::dpi::PhysicalSize::new(dimensions.x as f64, dimensions.y as f64);
        self.window.resize(size)
    }

    #[inline]
    fn poll_events(&mut self, events: &mut Vec<Event>) {
        let dims = self.dimensions_in_points();
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
