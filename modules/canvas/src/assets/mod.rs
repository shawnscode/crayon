mod font;
mod font_sys;

mod atlas;
mod atlas_sys;

pub mod errors;

pub use self::font::{Font, FontHandle};
pub use self::font_sys::FontSystem;

pub use self::atlas::{Atlas, AtlasFrame};
pub use self::atlas_sys::AtlasSystem;

pub(crate) struct CanvasAssets {
    pub fonts: FontSystem,
    pub atlas: AtlasSystem,
}