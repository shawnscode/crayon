#[macro_use]
extern crate crayon;
extern crate imgui;

pub mod canvas;
mod renderer;

pub mod prelude {
    pub use canvas::Canvas;
    pub use imgui::*;
}
