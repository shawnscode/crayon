//! Provides unified access to input devices across platforms.

mod keyboard;
mod mouse;
mod touchpad;

pub mod input;
pub use self::input::{InputSystem, InputSystemShared};

pub const MAX_TOUCHES: usize = 4;