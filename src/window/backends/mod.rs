mod headless;

use crate::errors::*;
use crate::math::prelude::Vector2;

use super::events::Event;

pub trait Visitor {
    fn show(&self);
    fn hide(&self);
    fn position(&self) -> Vector2<i32>;
    fn dimensions(&self) -> Vector2<u32>;
    fn device_pixel_ratio(&self) -> f32;
    fn resize(&self, dimensions: Vector2<u32>);
    fn poll_events(&mut self, events: &mut Vec<Event>);
    fn is_current(&self) -> bool;
    fn make_current(&self) -> Result<()>;
    fn swap_buffers(&self) -> Result<()>;
}

pub fn new_headless() -> Box<Visitor> {
    Box::new(self::headless::HeadlessVisitor {})
}

#[cfg(not(target_arch = "wasm32"))]
mod glutin;
#[cfg(not(target_arch = "wasm32"))]
pub use self::glutin::new;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use self::web::new;
