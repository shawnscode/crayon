extern crate crayon;
#[macro_use]
extern crate failure;

pub mod scene;

pub mod prelude {
    pub use scene::SceneGraph;
}
