#[cfg(not(target_arch = "wasm32"))]
mod glutin;
#[cfg(not(target_arch = "wasm32"))]
pub use self::glutin::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use self::web::*;
