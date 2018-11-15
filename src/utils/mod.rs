//! Commonly used utilities like pools, variants and so on.

#[macro_use]
pub mod handle;
pub mod data_buf;
pub mod double_buf;
pub mod handle_pool;
pub mod hash;
pub mod hash_value;
pub mod object_pool;
pub mod time;

pub mod prelude {
    pub use super::data_buf::{DataBuffer, DataBufferPtr};
    pub use super::double_buf::DoubleBuf;
    pub use super::handle::{Handle, HandleIndex, HandleLike};
    pub use super::handle_pool::HandlePool;
    pub use super::hash::{FastHashMap, FastHashSet};
    pub use super::hash_value::HashValue;
    pub use super::object_pool::ObjectPool;
    pub use super::time::Timestamp;
}
