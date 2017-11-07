//! Build-in assets including `Texture`, `Atlas`, `Shader` etc..

pub mod texture;
pub mod texture_sys;

pub use self::texture::Texture;
pub use self::texture_sys::TextureSystem;