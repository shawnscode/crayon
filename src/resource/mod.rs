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

use std::path::Path;
use std::error::Error;
use std::result::Result;

use futures;
use futures::{Async, Poll, Future};

/// The future version of resource.
pub struct ResourceFuture<T, E: Error>(futures::sync::oneshot::Receiver<Result<T, E>>);

impl<T, E> ResourceFuture<T, E>
    where E: Error + From<self::errors::Error>
{
    #[inline]
    pub fn new(rx: futures::sync::oneshot::Receiver<Result<T, E>>) -> Self {
        ResourceFuture(rx)
    }
}

impl<T, E> Future for ResourceFuture<T, E>
    where E: Error + From<self::errors::Error>
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(Async::Ready(x)) => Ok(Async::Ready(x?)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => {
                let err: self::errors::Error = self::errors::ErrorKind::FutureCanceled.into();
                Err(err.into())
            }
        }
    }
}

pub trait ResourceArenaLoader: Send + Sync + 'static {
    type Item: Send + Sync + 'static;
    type Error: Error + From<self::errors::Error> + Send;

    fn get(&self, _: &Path) -> Option<Self::Item> {
        None
    }

    fn insert(&self, path: &Path, bytes: &[u8]) -> Result<Self::Item, Self::Error>;
}

pub trait ResourceArenaMapper: Send + Sync + 'static {
    type Source: Send + Sync + 'static;
    type Item: Send + Sync + 'static;
    type Error: Error + From<self::errors::Error> + Send;

    fn map(&self, src: &Self::Source) -> Result<Self::Item, Self::Error>;
}