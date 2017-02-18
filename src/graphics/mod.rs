use utility::Handle;

pub mod color;
pub mod pipeline;
pub mod resource;
pub mod frame;
pub mod backend;

pub use self::color::Color;
pub use self::pipeline::RenderState;

impl_handle!(ViewHandle);
impl_handle!(PipelineHandle);
impl_handle!(FrameBufferHandle);
impl_handle!(VertexBufferHandle);
impl_handle!(IndexBufferHandle);

const MAX_ATTRIBUTES: usize = 16;