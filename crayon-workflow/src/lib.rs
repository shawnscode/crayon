//! # The Workflow Library.
//!
//! Builds a game project is not only about compile the source files into binary, but always comes with
//! numerous trivial tasks. For example, its common to handle the pre-processing of resources or archive
//! release package with some kind of shell scripts. In most senarios, its always tedious and error-prone.
//! To ease the development with crayon, its vital to address a flexible and robust workflow for these
//! works.
//!
//! The whole workflow mechanism is provided as a libaray to serve as backend for various kind of interfaces,
//! even by third parties or written in-house. You can find a build-in command line version named
//! [crayon-cli](https://github.com/shawnscode/crayon/tree/master/crayon-cli).
//!
//! ## Workspace
//!
//! To accomplish the goals, we introduces the concept of `workspace`, which defines where in your computer's
//! file system to store your project. There is also a `manifest`, just like what `Cargo` does, with various
//! bits of project information. We use the `manifest` to figure out the wanted behaviours when running
//! workflow tasks.
//!
//! Checkout the workspace module for more details.
//!
//! ## Resource
//!
//! Its hard to make a decent game, especially when you are dealing with massive resources. We introduce
//! `.meta` files to store the import settings for every tracking resources. They will be converted into
//! optimized runtime formats based on target platform when building the project.
//!
//! Checkout the resource module for more details.


#[macro_use]
extern crate error_chain;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate serde_json;
extern crate toml;
extern crate bincode;

extern crate image;
extern crate walkdir;
extern crate seahash;

#[macro_use]
extern crate nom;
extern crate libc;

extern crate crayon;

pub mod errors;
pub mod utils;
pub mod platform;
pub mod resource;
pub mod shaderc;
pub mod workspace;
pub mod prelude;