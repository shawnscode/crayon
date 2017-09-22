//! A stateless, layered, multithread graphics system with OpenGL backends.
//!
//! The management of graphics effects has become an important topic and key
//! feature of rendering engines. With the increasing number of effects it is
//! not sufficient anymore to only support them, but also to integrate them
//! into the rendering engine in a clean and extensible way.
//!
//! The goal of this work and simultaneously its main contribution is to design
//! and implement an advanced effects framework. Using this framework it should
//! be easy for further applications to combine several small effects like texture
//! mapping, shading and shadowing in an automated and transparent way and
//! apply them to any 3D model. Ad- ditionally, it should be possible to integrate
//! new effects and use the provided framework for rapid prototyping.

pub mod errors;
#[macro_use]
pub mod macros;
pub mod backend;

pub mod color;
pub use self::color::Color;

pub mod pipeline;
pub use self::pipeline::*;

pub mod resource;
pub use self::resource::*;

pub mod frame;
pub use self::frame::FrameTaskBuilder;

pub mod frontend;
pub use self::frontend::GraphicsFrontend;
pub use self::frontend::{ViewStateObject, ViewStateRef, ViewHandle};
pub use self::frontend::{PipelineStateObject, PipelineStateRef, PipelineStateHandle};
pub use self::frontend::{FrameBufferObject, FrameBufferRef, FrameBufferHandle};
pub use self::frontend::{VertexBufferObject, VertexBufferRef, VertexBufferHandle};
pub use self::frontend::{IndexBufferObject, IndexBufferRef, IndexBufferHandle};
pub use self::frontend::{TextureObject, TextureRef, TextureHandle};
pub use self::frontend::{RenderBufferObject, RenderBufferRef, RenderBufferHandle};