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

pub use self::resource::{ResourceSystem, ResourceSystemShared};

mod resource;

use std::sync::Arc;
use std::path::Path;

use futures;
use futures::{Async, Poll, Future};
use self::errors::*;

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

/// The future version of resource.
pub struct ResourceFuture<T>(futures::sync::oneshot::Receiver<Result<Arc<T>>>);

impl<T> Future for ResourceFuture<T> {
    type Item = Arc<T>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(Async::Ready(x)) => Ok(Async::Ready(x?)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => bail!(ErrorKind::FutureCanceled),
        }
    }
}

///
pub trait ExternalResourceSystem {
    type Item: Send + Sync + 'static;
    type Data: Resource;
    type Options: Send + Sync + Copy;

    fn load(&mut self,
            path: &Path,
            src: &Self::Data,
            options: Self::Options)
            -> self::errors::Result<Arc<Self::Item>>;

    fn unload_unused(&mut self);
}