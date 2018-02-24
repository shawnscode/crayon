//! The backend of renderer, which should be responsible for only one thing:
//! submitting draw-calls using low-level OpenGL graphics APIs.

pub mod errors;
pub mod capabilities;
pub mod device;
pub mod visitor;
pub mod frame;
