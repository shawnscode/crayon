//! A stateless, layered, multithread graphics system with OpenGL backends.
//!
//! # Graphics
//!
//! The management of graphics effects has become an important topic and key feature of
//! rendering engines. With the increasing number of effects it is not sufficient anymore
//! to only support them, but also to integrate them into the rendering engine in a clean
//! and extensible way.
//!
//! The goal of this work and simultaneously its main contribution is to design and
//! implement an advanced effects framework. Using this framework it should be easy for
//! further applications to combine several small effects like texture mapping, shading
//! and shadowing in an automated and transparent way and apply them to any 3D model.
//! Additionally, it should be possible to integrate new effects and use the provided
//! framework for rapid prototyping.
//!
//! ## Multi Platform
//!
//! Ideally, crayon should be able to run on macOS, windows and popular mobile-platforms.
//! There still are a huge number of performance and feature limited devices, so this
//! graphics module will always be limited by lower-end 3D APIs like OpenGL ES2.0.
//!
//! ## Stateless Pipeline
//!
//! Ordinary OpenGL application deals with stateful APIs. This means whenever you change
//! any state in the API for subsequent draw calls, this state change also affects draw
//! calls submitted at a later point in time.
//!
//! As a result of that, `PipelineStateObject` is introduced to encapsulate all stateful
//! things we need to configurate graphics pipeline. This would also enable us to easily
//! change the order of draw calls and get rid of redundant state changes.
//!
//! ## View
//!
//! All the real draw commands are executing delayed (and asynchronous if multi-thread
//! mode is enable), every draw calls user submitted are stored in a named _bucket_ with a
//! 64-bits _key_. The draw calls will be sorted based on the key before we kick commands
//! to GPU. Depending on where those bits are stored in the key, you can apply different
//! sorting criteria for the same array of draw calls.  Usually, a key encodes certain
//! data like distance, material, shader etc. in individual bits.
//!
//! We use `ViewStateObject` to represent a named _bucket_ mentioned above. You can also
//! configurate the targeting frame buffer and the clear flags on it.
//!
//! In case where order has to be preserved (for example in rendering GUIs), view can
//! be set to be in sequential order. Sequential order is less efficient, because it
//! doesn't allow state change optimization, and should be avoided when possible.
//!
//! ## Multi-thread
//!
//! In most cases, dividing OpenGL rendering across multiple threads will not result in
//! any performance improvement due the pipeline nature of OpenGL. What we are about
//! to do is actually exploiting parallelism in resource preparation, and provides a set of
//! multi-thread friendly APIs.

mod backend;
mod uniform_variable;
mod color;

pub mod view;
pub mod pipeline;
pub mod mesh;
pub mod texture;

pub mod errors;
#[macro_use]
pub mod macros;
pub mod frame;
pub mod graphics;

pub use self::view::*;
pub use self::pipeline::*;
pub use self::mesh::*;
pub use self::texture::*;

pub use self::color::*;
pub use self::uniform_variable::*;
pub use self::frame::DrawCallBuilder;

pub use self::graphics::GraphicsSystem;