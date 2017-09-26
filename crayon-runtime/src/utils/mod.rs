#[macro_use]
pub mod handle;
pub mod handle_set;
pub mod handle_object_set;
pub mod hash;
pub mod variant;

mod finally;

pub use self::handle::{Handle, HandleIndex};
pub use self::handle_set::{HandleSet, HandleIter};
pub use self::handle_object_set::HandleObjectSet;
pub use self::finally::{finally, finally_with};
pub use self::hash::{hash, HashValue};
pub use self::variant::{VariantChar, VariantStr};