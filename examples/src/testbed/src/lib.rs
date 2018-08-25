extern crate crayon;
pub use crayon::*;

extern crate crayon_3d;
#[macro_use]
extern crate crayon_imgui;
pub use crayon_imgui::*;

extern crate env_logger;

pub mod console;

pub fn settings<T1, T2>(titile: T1, dimesions: T2) -> crayon::application::Settings
where
    T1: Into<String>,
    T2: Into<crayon::math::Vector2<u32>>,
{
    ::env_logger::init();

    let mut params = crayon::application::Settings::default();
    params.window.title = titile.into();
    params.window.size = dimesions.into();

    let args: Vec<String> = ::std::env::args().collect();
    params.headless = args.len() > 1 && args[1] == "headless";
    params
}

pub fn find_res_dir() -> crayon::res::vfs::DiskFS {
    use std::path::Path;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let search_dirs = [
        root.join("resources"),
        root.parent().unwrap().join("resources"),
    ];

    for v in &search_dirs {
        if v.is_dir() && v.join(crayon::res::manifest::NAME).exists() {
            return crayon::res::vfs::DiskFS::new(v).unwrap();
        }
    }

    panic!("Could not found compiled resources.");
}

pub mod prelude {
    pub use super::console::ConsoleCanvas;

    pub use crayon_3d::prelude::*;

    // pub use crayon_imgui::prelude::TextureHandle as ImGuiTextureHandle;
    // pub use crayon_imgui::prelude::*;

    pub use crayon::errors::*;
    pub use crayon::prelude::*;
    pub use crayon::video::assets::prelude::*;
}
