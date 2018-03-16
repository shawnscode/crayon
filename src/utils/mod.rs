//! Commonly used utilities like pools, variants and so on.

#[macro_use]
pub mod handle;
pub mod handle_pool;
pub mod hash;
pub mod hash_value;
pub mod variant;
pub mod data_buf;

mod finally;
mod color;
mod rect;

pub use self::handle::{Handle, HandleIndex};
pub use self::handle_pool::HandlePool;
pub use self::finally::{finally, finally_with};
pub use self::hash::hash;
pub use self::hash_value::HashValue;
pub use self::variant::{VariantChar, VariantStr};
pub use self::data_buf::{DataBuffer, DataBufferPtr};
pub use self::rect::*;
pub use self::color::*;
