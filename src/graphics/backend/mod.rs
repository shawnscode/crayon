//! The backend of renderer, which should be responsible for only one thing:
//! submitting draw-calls using low-level OpenGL graphics APIs.

pub mod capabilities;
pub mod device;
pub mod errors;
pub mod frame;
pub mod visitor;
