#[macro_use]
use utility::Handle;

pub mod color;
pub mod pipeline;
pub mod frame;

pub use self::color::Color;
pub use self::pipeline::RenderState;

impl_handle!(ViewHandle);
impl_handle!(PipelineHandle);
impl_handle!(FrameBufferHandle);
impl_handle!(VertexBufferHandle);
impl_handle!(IndexBufferHandle);
