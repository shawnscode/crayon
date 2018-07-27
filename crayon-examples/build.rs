extern crate crayon_workflow;

use crayon_workflow::assets::TextureCompiler;
use crayon_workflow::database::Database;
use std::path::Path;

fn main() {
    let cwd = ::std::env::current_dir().unwrap();
    let out_dir = ::std::env::var("OUT_DIR").unwrap();

    let mut database = Database::new();
    database.register_extensions(TextureCompiler {}, &["png"]);

    let src = cwd.join("assets");
    let dst = Path::new(&out_dir).join("assets");
    database.build(&src, &dst).unwrap();

    println!("WD {:?}", dst);
}
