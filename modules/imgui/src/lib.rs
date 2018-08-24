#[macro_use]
extern crate crayon;

extern crate imgui;
#[doc(hidden)]
pub use imgui::*;

pub mod canvas;
mod renderer;

pub use self::canvas::Canvas;
