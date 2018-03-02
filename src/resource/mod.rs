//! The standardized interface to load data asynchronously from the `Filesystem`, and
//! provides utilities for modules to implement their own local resource management.
//!
//! This is a very general overview of the resource management philosophy in `Crayon`.
//! Modules are completely free to implement their own resource management and don’t need
//! to adhere to this basic philosophy.
//!
//! For specific information on how to create and use resources, please read the
//! particular module documentations.
//!
//! # Resource Management
//!
//! A resource is a very slim proxy object that adds a standardized interface for creation,
//! destruction, sharing and lifetime management to some external object or generally
//! ‘piece of data'.
//!
//! Its recommanded to use a unique `Handle` object to represent a resource object safely.
//! This approach has several advantages, since it helps for saving state externally. E.G.:
//!
//! 1. It allows for the resource to be destroyed without leaving dangling pointers.
//! 2. Its perfectly safe to store and share the `Handle` even the underlying resource is
//! loading on the background thread.
//!
//! In some systems, actual resource objects are private and opaque, application will usually
//! not have direct access to a resource object in form of reference.
//!
//! ## Sharing
//!
//! Safe resource sharing is implemented through resource's `Location`. A `Location` object has (
//! in its usual form) a `Path` slice that serves as a human-readable identifier, and (for
//! resources that are loaded from a filesystem) also as an URL.
//!
//! And besides the `Path` part, there is a additional `Signature` field in `Location`. A
//! `Signature` is usually a integer which is used to restrict sharing. Two `Location`s are
//! only identical if both the path and signature match. This can be used to suppress resource
//! sharing even if the path (e.g. the filename) of two `Location`s matches.
//!
//! ```rust,ignore
//! let l1 = Location::token(0, "/path/to/res");
//! let l2 = Location::token(0, "/path/to/res");
//! assert!(l1 == l2);
//!
//! let l3 = Location::token(1, "/path/to/res");
//! assert!(l1 != l3);
//!
//! let l4 = Location::shared("/path/to/res");
//! let l5 = Location::shared("/path/to/res");
//! assert!(l4 == l5);
//! assert!(l4 != l1);
//! ```
//!
//! There is one special `Unique` signature which disables sharing of a resource completely,
//! and always makes the resource object unique, no matter how many other shared or non-shared
//! resources with the same name exist, this is most useful to enforce private ownership of
//! a resource without having to care about name collisions.
//!
//! ```rust,ignore
//! let l1 = Location::unique("/path/to/res");
//! let l2 = Location::unique("/path/to/res");
//! assert!(l1 != l2);
//! ```
//!
//! ## Lifetime (TODO)
//!
//! ## Asynchronization (TODO)
//!

pub mod errors;
pub mod filesystem;
pub mod utils;

mod service;
pub use self::service::{ResourceFS, ResourceSystem, ResourceSystemShared, ResourceTask};

pub mod prelude {
    pub use super::{ResourceFS, ResourceSystem, ResourceSystemShared, ResourceTask};
    pub use super::filesystem::{DirectoryFS, ZipFS};
    pub use super::utils::location::Location;
}
