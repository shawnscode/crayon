//! The standardized interface for loading, sharing and lifetime management of resources.
//!
//! # Resource
//!
//! A resource is a very slim proxy object that adds a standardized interface for
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
//!
//! ## Sharing & Lifetime
//!
//! Resource sharing is implemented through _Path_, which is human-readable and could
//! serves as an URL when loading from filesystem. Whenever user load a resource
//! from `ResourceSystem`, a thread-safe shared-ptr will be returned. It guarantees that
//! resources will not be freed until user disposed the shared-ptr and no duplicated copy
//! with same _Path_ identifier.

pub mod errors;
pub mod filesystem;
pub mod cache;
pub mod arena;
pub mod resource;
pub mod assets;

pub use self::resource::{ResourceSystem, ResourceSystemShared, ResourceFrameInfo, ResourceFuture};

use std::sync::{Arc, RwLock};

/// The trait `` is a simple place-holder, which produces a
/// continuous index address. It always should be implemented by using
/// macro `declare_resource`.
/// Provides some essential informations of resource.
pub trait Resource {
    fn size(&self) -> usize;
}

/// The thread-safe and shared ptr alias to resource.
pub type Ptr<T> = Arc<RwLock<T>>;

/// A `ResourceParser` provides a conversion from bytes to asset data.
pub trait ResourceParser {
    type Item: Resource + Send + Sync + 'static;

    fn parse(bytes: &[u8]) -> self::errors::Result<Self::Item>;
}
