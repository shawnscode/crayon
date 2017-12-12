#[macro_use]
extern crate crayon;

#[macro_use]
extern crate error_chain;

extern crate imgui;

pub mod errors;
pub mod renderer;
pub mod canvas;
pub mod prelude;

use crayon::application;
use errors::*;

pub fn new(ctx: &application::Context) -> Result<(canvas::Canvas, renderer::Renderer)> {
    let mut imgui = imgui::ImGui::init();
    imgui.set_ini_filename(None);

    let renderer = renderer::Renderer::new(ctx, &mut imgui)?;
    let canvas = canvas::Canvas::new(imgui)?;

    Ok((canvas, renderer))
}