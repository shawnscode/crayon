#[macro_use]
extern crate error_chain;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;
extern crate toml;

extern crate image;
extern crate walkdir;
extern crate seahash;

extern crate crayon;

pub mod errors;
pub mod manifest;
pub mod platform;
pub mod resource;
pub mod serialization;

pub use manifest::Manifest;
pub use errors::*;
pub use resource::{Resource, ResourceMetadata, TextureMetadata};