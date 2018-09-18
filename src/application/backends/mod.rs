mod headless;
pub use self::headless::HeadlessVisitor;

use application::events::Event;
use application::settings::WindowParams;
use errors::*;
use math;

pub trait Visitor {
    fn show(&self);
    fn hide(&self);
    fn position_in_points(&self) -> math::Vector2<i32>;
    fn dimensions_in_points(&self) -> math::Vector2<u32>;
    fn hidpi(&self) -> f32;
    fn resize(&self, dimensions: math::Vector2<u32>);
    fn poll_events(&mut self, events: &mut Vec<Event>);
    fn is_current(&self) -> bool;
    fn make_current(&self) -> Result<()>;
    fn swap_buffers(&self) -> Result<()>;
}

mod glutin;

pub fn new(params: WindowParams) -> Result<Box<Visitor>> {
    let visitor = glutin::GlutinVisitor::new(params)?;
    Ok(Box::new(visitor))
}
