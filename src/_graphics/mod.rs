//! #### Graphics Subsystem
//! Graphics is the most fundamental subsystem of [Lemon3d](http://github.com/kayak233/lemon3d).
//! It was degisned to provide a set of stateless and high-performance
//! graphics APIs based on OpenGL.

pub mod color;
pub mod state;
pub mod buffer;
pub mod backend;
pub mod shader;
pub mod drawcall;
pub mod frontend;
pub mod graphics;

mod frame;

pub use self::frontend::ViewObject;
pub use self::graphics::Graphics;

const MAX_VERTEX_ATTRIBUTES: usize = 8;