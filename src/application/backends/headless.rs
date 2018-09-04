use math;

use application::events::Event;
use errors::*;

use super::Visitor;

pub struct HeadlessVisitor {}

impl Visitor for HeadlessVisitor {
    #[inline]
    fn show(&self) {}

    #[inline]
    fn hide(&self) {}

    #[inline]
    fn position_in_points(&self) -> math::Vector2<i32> {
        (0, 0).into()
    }

    #[inline]
    fn dimensions_in_points(&self) -> math::Vector2<u32> {
        (0, 0).into()
    }

    #[inline]
    fn hidpi(&self) -> f32 {
        1.0
    }

    #[inline]
    fn resize(&self, _: math::Vector2<u32>) {}

    #[inline]
    fn poll_events(&mut self, _: &mut Vec<Event>) {}

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
}
