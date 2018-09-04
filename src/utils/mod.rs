//! Commonly used utilities like pools, variants and so on.

#[macro_use]
pub mod handle;
pub mod data_buf;
pub mod handle_pool;
pub mod hash;
pub mod hash_value;
pub mod object_pool;
pub mod variant;

pub use self::data_buf::{DataBuffer, DataBufferPtr};
pub use self::handle::{Handle, HandleIndex};
pub use self::handle_pool::HandlePool;
pub use self::hash_value::HashValue;
pub use self::variant::{VariantChar, VariantStr};
