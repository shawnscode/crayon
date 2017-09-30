//! # Resource
//!
//! A resource is a very slim proxy object that aadds a standardized interface for
//! creation, destruction, sharing and lifetime management ot some other external
//! object or generally 'piece of data'.
//!
//! ## Virtual Filesystem
//!
//! The virtual file-system module allows to load data asynchronously from local host disk,
//! zip file, or other places that implemented `Filesystem`. Note that it does NOT support
//! general filesystem operations that might be required by other common application types,
//! like directory operations etc..
//!
//! Most operations of `Filesystem` are actually done on a sperate thread, and returns a
//! _future_.
//!
//! ## Formats
//!
//! Resource comes with different formats, you can load resource with an intermediate
//! format, or your own parser by implementing trait `ResourceParser`.

pub mod errors;
pub mod filesystem;
pub mod cache;

pub use self::filesystem::{Filesystem, FilesystemDriver};
pub use self::cache::Cache;