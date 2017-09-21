use utility::Handle;

pub mod color;
pub mod pipeline;
pub mod resource;
pub mod frame;
pub mod graphics;
pub mod errors;
#[macro_use]
pub mod macros;
pub mod backend;

pub use self::resource::*;
pub use self::pipeline::*;
pub use self::color::Color;
pub use self::graphics::{Graphics, ViewStateRef, PipelineStateRef, FrameBufferRef, TextureRef,
                         RenderBufferRef, VertexBufferRef, IndexBufferRef};
pub use self::frame::FrameTaskBuilder;

impl_handle!(ViewHandle);
impl_handle!(PipelineStateHandle);
impl_handle!(VertexBufferHandle);
impl_handle!(IndexBufferHandle);
impl_handle!(TextureHandle);
impl_handle!(RenderBufferHandle);
impl_handle!(FrameBufferHandle);