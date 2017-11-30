//! # Crayon's Canvas Module
//!
//! A _Canvas_ is the area that all UI nodes should be inside.

#[macro_use]
extern crate crayon;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;

extern crate rusttype;
extern crate unicode_normalization;

pub mod assets;
pub mod errors;
pub mod renderer;
pub mod prelude;

mod canvas;
mod node;
mod element;
mod layout;