//! The backend of renderer, which should be responsible for only one
//! thing: submitting draw-calls using low-level OpenGL graphics APIs.

pub mod errors;
pub mod context;
pub mod backend;

pub use self::errors::*;

use super::*;
use super::resource::*;
use super::pipeline::*;

pub trait Context {
    /// Swaps the buffers in case of double or triple buffering.
    ///
    /// **Warning**: if you enabled vsync, this function will block until the
    /// next time the screen is refreshed. However drivers can choose to
    /// override your vsync settings, which means that you can't know in advance
    /// whether swap_buffers will block or not.
    fn swap_buffers(&self) -> Result<()>;

    /// Returns true if this context is the current one in this thread.
    fn is_current(&self) -> bool;

    /// Set the context as the active context in this thread.
    unsafe fn make_current(&self) -> Result<()>;
}

/// Render state managements.
pub trait RenderStateVisitor {
    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    unsafe fn set_viewport(&mut self, position: (u16, u16), size: (u16, u16)) -> Result<()>;

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_face_cull(&mut self, face: CullFace) -> Result<()>;

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face(&mut self, front: FrontFaceOrder) -> Result<()>;

    /// Specify the value used for depth buffer comparisons.
    unsafe fn set_depth_test(&mut self, comparsion: Comparison) -> Result<()>;

    /// Enable or disable writing into the depth buffer.
    ///
    /// Optional `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write(&mut self, enable: bool, offset: Option<(f32, f32)>) -> Result<()>;

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(&mut self,
                              blend: Option<(Equation, BlendFactor, BlendFactor)>)
                              -> Result<()>;

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(&mut self,
                              red: bool,
                              green: bool,
                              blue: bool,
                              alpha: bool)
                              -> Result<()>;
}

/// Graphics resource managements.
pub trait ResourceStateVisitor {
    /// Initialize buffer, named by `handle`, with optional initial data.
    fn create_vertex_buffer(&mut self,
                            handle: VertexBufferHandle,
                            buffer: Resource,
                            hint: ResourceHint,
                            size: u32,
                            data: Option<&[u8]>)
                            -> Result<()>;

    /// Update named dynamic `MemoryHint::Dynamic` buffer.
    ///
    /// Optional `offset` to specifies the offset into the buffer object's data
    /// store where data replacement will begin, measured in bytes.
    fn update_vertex_buffer(&mut self,
                            handle: VertexBufferHandle,
                            offset: u32,
                            data: &[u8])
                            -> Result<()>;

    /// Free named buffer object.
    fn free_vertex_buffer(&mut self, handle: VertexBufferHandle) -> Result<()>;

    /// Initializes named program object. A program object is an object to
    /// which shader objects can be attached. Vertex and fragment shader
    /// are minimal requirement to build a proper program.
    fn create_program(&mut self, handle: PipelineHandle, vs_src: &str, fs_src: &str) -> Result<()>;

    /// Free named program object.
    fn free_program(&mut self, handle: PipelineHandle) -> Result<()>;
}

pub trait RasterizationStateVisitor {
    /// Clear any or all of rendertarget, depth buffer and stencil buffer.
    fn clear(&self,
             color: Option<[f32; 4]>,
             depth: Option<f32>,
             stencil: Option<i32>)
             -> Result<()>;

    /// Bind a named vertex buffer object.
    fn set_vertex_buffer(&mut self, handle: VertexBufferHandle) -> Result<()>;

    /// Bind a named program object.
    fn set_program(&mut self, handle: PipelineHandle) -> Result<()>;

    /// Bind a named uniform.
    fn set_uniform(&mut self, name: u64, variable: &UniformVariable) -> Result<()>;

    /// Commit render primitives from binding data.
    fn commit(primitive: Primitive, from: u32, len: u32) -> Result<()>;
}