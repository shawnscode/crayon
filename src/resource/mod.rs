//! The standardized interface for loading, sharing and lifetime management of resources.
//!
//! # Resource
//!
//! A resource is a very slim proxy object that adds a standardized interface to
//! some other external object or generally 'piece of data'.

pub mod errors;
pub mod filesystem;
pub mod cache;

pub use self::resource::{ResourceSystem, ResourceSystemShared};

mod resource;

use std::sync::Arc;
use std::path::Path;

use futures;
use futures::{Async, Poll, Future};
use self::errors::*;

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

pub trait ResourceArenaLoader: Send + Sync + 'static {
    type Item: Send + Sync + 'static;

    fn get(&self, path: &Path) -> Option<Arc<Self::Item>>;
    fn parse(&self, bytes: &[u8]) -> Result<Self::Item>;
    fn insert(&self, path: &Path, item: Arc<Self::Item>);
}

pub trait ResourceArenaMapper: Send + Sync + 'static {
    type Source: Send + Sync + 'static;
    type Item: Send + Sync + 'static;

    fn map(&self, src: &Self::Source) -> Result<Arc<Self::Item>>;
}