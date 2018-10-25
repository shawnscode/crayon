mod headless;

use application::events::Event;
use errors::*;
use math::prelude::Vector2;

pub trait Visitor {
    fn show(&self);
    fn hide(&self);
    fn position_in_points(&self) -> Vector2<i32>;
    fn dimensions_in_points(&self) -> Vector2<u32>;
    fn hidpi(&self) -> f32;
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
pub use self::glutin::{new, sys};

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use self::web::{new, sys};
