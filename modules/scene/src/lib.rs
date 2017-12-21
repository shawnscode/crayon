//! _Crayon-Scene_ is a Rust library for building 2D/3D applications. Its based on
//! [crayon](https://github.com/shawnscode/crayon), and aims to be intuitive and
//! easy for user.

#[macro_use]
extern crate crayon;
#[macro_use]
extern crate error_chain;

pub mod errors;
pub mod node;
pub mod transform;
pub mod camera;
pub mod light;
pub mod scene;

pub mod prelude;