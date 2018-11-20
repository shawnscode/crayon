pub mod clip;
pub mod clip_loader;

pub mod prelude {
    pub use super::clip::{AudioClip, AudioClipHandle};
    pub use super::clip_loader::AudioClipLoader;
}
