//! The backend of renderer, which should be responsible for only one
//! thing: submitting draw-calls using low-level OpenGL graphics APIs.
use utility::Handle;

use super::buffer;
use super::buffer::*;

use super::state;
use super::state::*;

use super::Primitive;

pub mod cast;
pub mod device;

const MAX_BUFFERS: usize = 1024;
const MAX_PROGRAMS: usize = 64;

/// OpenGL errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    InvalidFramebufferOperation,
    OutOfMemory,
    UnknownError,
}

/// Render state managements.
pub trait RenderState {
    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    unsafe fn set_viewport(&mut self, position: (u32, u32), size: (u32, u32));

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_face_cull(&mut self, face: CullFace);

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face(&mut self, front: FrontFaceOrder);

    /// Specify the value used for depth buffer comparisons.
    unsafe fn set_depth_test(&mut self, comparsion: Comparison);

    /// Enable or disable writing into the depth buffer.
    ///
    /// Optional `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write(&mut self, enable: bool, offset: Option<(f32, f32)>);

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(&mut self,
                              enable: bool,
                              equation: Equation,
                              src: BlendFactor,
                              dst: BlendFactor);

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(&mut self, red: bool, green: bool, blue: bool, alpha: bool);
}

/// Graphics resource managements.
pub trait ResourceState {
    /// Initialize buffer, named by `handle`, with optional initial data.
    unsafe fn create_buffer(&mut self,
                            handle: Handle,
                            buffer: Buffer,
                            hint: BufferHint,
                            size: usize,
                            data: Option<&[u8]>);

    /// Update named dynamic `MemoryHint::Dynamic` buffer.
    ///
    /// Optional `offset` to specifies the offset into the buffer object's data
    /// store where data replacement will begin, measured in bytes.
    unsafe fn update_buffer(&mut self, handle: Handle, data: &[u8], offset: u32);

    /// Free named buffer object.
    unsafe fn free_buffer(&mut self, handle: Handle);

    /// Initializes named program object. A program object is an object to
    /// which shader objects can be attached. Vertex and fragment shader
    /// are minimal requirement to build a proper program.
    unsafe fn create_program(&mut self,
                             handle: Handle,
                             vs_src: &str,
                             fs_src: &str,
                             gs_src: Option<&str>);

    /// Free named program object.
    unsafe fn free_program(&mut self, handle: Handle);
}

pub trait RasterizationState {
    /// Clear any or all of rendertarget, depth buffer and stencil buffer.
    unsafe fn clear(&self, color: Option<[f32; 4]>, depth: Option<f32>, stencil: Option<i32>);

    /// Bind a named buffer object.
    unsafe fn bind_buffer(&mut self, handle: Handle);

    /// Bind a named program object.
    unsafe fn bind_program(&mut self, handle: Handle);

    /// Bind a named texture object into sampler unit.
    unsafe fn bind_texture(&mut self, handle: Handle, unit: u32);

    /// Commit render primitives from binding data.
    unsafe fn commit(primitive: Primitive, from: u32, len: u32);
}