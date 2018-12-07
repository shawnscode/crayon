use crate::errors::*;
use crate::math::prelude::Vector2;

use super::super::events::Event;
use super::Visitor;

pub struct HeadlessVisitor {}

impl Visitor for HeadlessVisitor {
    #[inline]
    fn show(&self) {}

    #[inline]
    fn hide(&self) {}

    #[inline]
    fn position(&self) -> Vector2<i32> {
        (0, 0).into()
    }

    #[inline]
    fn dimensions(&self) -> Vector2<u32> {
        (0, 0).into()
    }

    #[inline]
    fn device_pixel_ratio(&self) -> f32 {
        1.0
    }

    #[inline]
    fn resize(&self, _: Vector2<u32>) {}

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
