extern crate crayon;
extern crate crayon_imgui;
extern crate crayon_workflow;
#[macro_use]
extern crate failure;

pub mod console;
pub mod errors;

pub fn build(force: bool) -> ::std::path::PathBuf {
    use crayon_workflow::assets::TextureCompiler;
    use crayon_workflow::database::Database;

    let cwd = ::std::env::current_dir().unwrap();
    let src = cwd.join("assets");
    let dst = cwd.join("assets/compiled");

    if dst.is_dir() && !force {
        return dst;
    }

    let mut database = Database::new();
    database.register_extensions(TextureCompiler {}, &["png"]);
    database.build(&src, &dst).unwrap();

    dst
}

pub mod prelude {
    pub use super::console::ConsoleCanvas;
    pub use super::errors::*;
    pub use crayon_imgui::prelude::*;
}
