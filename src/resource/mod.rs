//! The standardized interface to load data asynchronously from the `Filesystem`, and
//! provides utilities for modules to implement their own local resource management.
//!
//! # Resource Management
//!
//! This is a very general overview of the resource management philosophy in `Crayon`.
//! Modules are completely free to implement their own resource management and don’t need
//! to adhere to this basic philosophy.
//!
//! For specific information on how to create and use resources, please read the
//! particular module documentations.
//!
//! A resource is a very slim proxy object that adds a standardized interface for creation,
//! destruction, sharing and lifetime management to some external object or generally
//! ‘piece of data'.
//!
//! Actual resource objects are private and opaque, application will usually not have
//! direct access to a resource object in form of reference. And a unique `Handle` object
//! is used to represent a resource object safely.
//!
//! This approach has several advantages, since it helps for saving state externally. E.G.:
//!
//! 1. It allows for the resource to be destroyed without leaving dangling pointers.
//! 2. Its perfectly safe to store and share the `Handle` even the underlying resource is
//! loading on the background thread.
//!
//! ## Lifetime Management
//!
//! Its common to manage lifetime with reference-counting technique. It kinda worked ok,
//! but implementing reference-counting for resource became complex and ugly because of
//! dependencies between resources.
//!
//! If you look at typical scenarios of game applications, it is apparent that resources
//! are often created and destroyed in batches or 'onion skin' layers. Some basic
//! resources need to exist during the whole lifetime of the application, other resources
//! are created at the start of a scene, and destroyed when the scene is left, or when a
//! sub-module is created or destroyed.
//!
//! As a result of that, reference-counting has been replaced with `ResourceLabel` and
//! batch-destruction by matching resource labels.
//!
//! All created resources are attached with a instance of `ResourceLabel`. When the batch
//! of resources is no longer needed, a single `delete` call which takes a resource label
//! as arguments destroys all resources matching the label at once.
//!
//! Any code still trying to use a resource `Handle` from this batch will _fail_. What _fail_
//! exactly means depends on how an invalid `Handle` is treated by the specific module. But
//! in general, it should NOT causes any low-level memory crashes, instead of throw an error,
//! or silently ignore the invalid `Handle`.
//!
//! ## Asynchronization (TODO)
//!
//! ## Sharing (TODO)

pub mod errors;
pub mod filesystem;
pub mod cache;

mod resource;
pub use self::resource::{ResourceSystem, ResourceSystemShared, ResourceAsyncLoader};