#[macro_use]
extern crate crayon;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;

extern crate rusttype;
extern crate lyon;

pub mod assets;
pub mod errors;
pub mod renderer;
pub mod prelude;

mod canvas;
mod node;
mod element;
mod layout;

// use crayon;
// pub fn register(resource: &mut crayon::resource::ResourceSystem) {
//     resource
// }