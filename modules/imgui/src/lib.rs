#[macro_use]
extern crate crayon;

#[macro_use]
extern crate error_chain;

extern crate imgui;

pub mod errors;
pub mod prelude;

mod renderer;
mod canvas;
pub use self::renderer::Renderer;
pub use self::canvas::Canvas;

pub use crayon::application;
use errors::*;

pub fn new(ctx: &application::Context) -> Result<(canvas::Canvas, renderer::Renderer)> {
    let mut imgui = imgui::ImGui::init();
    imgui.set_ini_filename(None);

    let renderer = renderer::Renderer::new(ctx, &mut imgui)?;
    let canvas = canvas::Canvas::new(imgui)?;

    Ok((canvas, renderer))
}