//! The `ResourceSystem` provides standardized interface to load data asynchronously from various
//! filesystem, and some utilities for modules to implement their own local resource management.
//!
//! To understand how to properly manage data in _crayon_, its important to understand how _crayon_
//! identifies and serializes data. The first key point is the distinction between _asset_s and
//! _resource_s.
//!
//! # Asset
//!
//! An asset is a file on disk, such like textures, 3D models, or audio clips. Since assets might
//! be be modified by artiest continuous, its usually stored in formats which could producing and
//! editing by authoring tools directly. Its always trival and error-prone to load and managet
//! assets directly at runtime.
//!
//! # Resource
//!
//! A _resource_ is a abstraction of some `piece of data` that are fully prepared for using at runtime.
//! We are providing a command line tool [crayon-cli](https://github.com/shawnscode/crayon-tools) that
//! automatically compiles assets into resources for runtime.
//!
//! ## UUID
//!
//! An asset can produces multiple resources eventually. For example, `FBX` file can have multiple
//! models, and it can also contains a spatial description of objects. For every resource that an
//! asset might produces, a universal-uniqued id (UUID) is assigned to it. UUIDs are stored in .meta
//! files. These .meta files are generated when _crayon-cli_ first imports an asset, and are stored
//! in the same directory as the asset.
//!
//! ## Location
//!
//! User could locates a `resource` with `Location`. A `Location` consists of two parts, a virtual
//! filesystem prefix and a readable identifier of resource.
//!
//! For example, let's say a game has all its textures in a special asset subdirectory called
//! `resources/textures`. The game would define an virtual filesystem called `res:` pointing to that
//! directory, and texture location would be defined like this:
//!
//! ```sh
//! "res:textures/crate.png"
//! ```
//!
//! Before those textures are actually loaded, the `res:` prefix is replaced with an absolute directory
//! location, and the readable path is replaced with the actual local path (which are usually the hex
//! representation of UUID).
//!
//! ```sh
//! "res:textures/crate.png" => "/Applications/My Game/resources/textures/2943B9386A274730A50702A904F384D5"
//! ```
//!
//! This makes it easier to load data from other places than the local hard disc, like web servers,
//! communicating with HTTP REST services or implementing more exotic ways to load data.
//!
//! # Virtual Filesystem (VFS)
//!
//! The `ResourceSystem` allows load data asynchronously from web servers, the local host
//! filesystem, or other places if extended by pluggable `VFS`.
//!
//! The `VFS` trait has a pretty simple interface, since it should focus on games that load
//! data asynchronously. A trival `Directory` is provided to supports local host filesystem.
//! And it should be easy to add features like compression and encrpytion.
//!
//! ## Manifest
//!
//! Every VFS should have a `Manifest` file which could be used to locate resources in actual path
//! from general UUID or readable identifier. The `Manifest` file is generated after the build
//! process of `crayon-cli`.
//!
//! # Registry
//!
//! The `Registry` is a standardized resource manager that defines a set of interface for creation,
//! destruction, sharing and lifetime management. It is used in all the built-in crayon modules.
//!
//! ## Handle
//!
//! We are using a unique `Handle` object to represent a resource object safely. This approach
//! has several advantages, since it helps for saving state externally. E.G.:
//!
//! 1. It allows for the resource to be destroyed without leaving dangling pointers.
//! 2. Its perfectly safe to store and share the `Handle` even the underlying resource is
//! loading on the background thread.
//!
//! In some systems, actual resource objects are private and opaque, application will usually
//! not have direct access to a resource object in form of reference.
//!
//! ## Ownership & Lifetime
//!
//! For the sake of simplicity, the refenerce-counting technique is used for providing shared ownership
//! of a resource.
//!
//! Everytime you create a resource at runtime, the `Registry` will increases the reference count of
//! the resource by 1. And when you are done with the resource, its the user's responsibility to
//! drop the ownership of the resource. And when the last ownership to a given resource is dropped,
//! the corresponding resource is also destroyed.
//!

pub mod manifest;
pub mod request;
pub mod shortcut;
pub mod url;
pub mod utils;
pub mod vfs;

mod ctx;
mod worker;

pub use self::ctx::{discard, setup, valid};
pub use self::ctx::{exists, find, load, load_from, resolve};
pub use self::ctx::{load_from_with_callback, load_with_callback};
