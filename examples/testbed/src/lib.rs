extern crate crayon;
extern crate crayon_imgui;

pub mod console;

pub fn settings<T1, T2>(titile: T1, dimesions: T2) -> crayon::application::Settings
where
    T1: Into<String>,
    T2: Into<crayon::math::Vector2<u32>>,
{
    let mut params = crayon::application::Settings::default();
    params.window.title = titile.into();
    params.window.size = dimesions.into();

    let args: Vec<String> = ::std::env::args().collect();
    params.headless = args.len() > 1 && args[1] == "headless";

    params
}

pub mod prelude {
    pub use super::console::ConsoleCanvas;
    pub use crayon_imgui::prelude::*;
}
