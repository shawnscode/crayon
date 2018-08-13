extern crate crayon;
extern crate crayon_imgui;
#[macro_use]
extern crate failure;

pub mod console;
pub mod errors;

pub mod prelude {
    pub use super::console::ConsoleCanvas;
    pub use super::errors::*;
    pub use crayon_imgui::prelude::*;
}
