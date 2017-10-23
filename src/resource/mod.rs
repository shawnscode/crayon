//! The standardized interface for loading, sharing and lifetime management of resources.
//!
//! # Resource
//!
//! A resource is a very slim proxy object that adds a standardized interface to
//! some other external object or generally 'piece of data'.

pub mod errors;
pub mod filesystem;
pub mod cache;
pub mod arena;

mod resource;

pub use self::resource::{ResourceSystem, ResourceSystemShared, ResourceFrameInfo, ResourceFuture};

/// A slim proxy trait that adds a standardized interface of resource.
pub trait Resource: Send + Sync + 'static {
    fn size(&self) -> usize;
}

/// Resources comes with various formats usually, we could introduce a conversion
/// from plain bytes to resource instance by implementing trait `ResourceParser`.
pub trait ResourceParser {
    type Item: Resource;

    fn parse(bytes: &[u8]) -> self::errors::Result<Self::Item>;
}