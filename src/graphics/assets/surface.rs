//! Named bucket of draw calls with the wrapping of rendering operations to a render
//! target, clearing, MSAA resolving and so on.

use utils::Color;
use graphics::MAX_FRAMEBUFFER_ATTACHMENTS;
use graphics::assets::texture::{RenderBufferHandle, RenderTextureHandle};
use graphics::errors::*;

/// SurfaceObject wraps rendering operations to a render-target. Likes clearing, MSAA
/// resolves, etc..
///
/// It also plays as the named bucket of draw commands. Drawcalls inside `Surface` are
/// sorted before submitting to underlaying OpenGL. In case where order has to be
/// preserved (for example in rendering GUIs), view can be set to be in sequential order.
/// Sequential order is less efficient, because it doesn't allow state change optimization,
/// and should be avoided when possible.
#[derive(Debug, Copy, Clone)]
pub struct SurfaceSetup {
    pub(crate) framebuffer: Option<FrameBufferHandle>,
    pub(crate) clear_color: Option<Color>,
    pub(crate) clear_depth: Option<f32>,
    pub(crate) clear_stencil: Option<i32>,
    pub(crate) order: u64,
    pub(crate) sequence: bool,
    pub(crate) viewport: ((f32, f32), (f32, f32)),
}

impl Default for SurfaceSetup {
    fn default() -> Self {
        SurfaceSetup {
            framebuffer: None,
            clear_color: Some(Color::black()),
            clear_depth: Some(1.0),
            clear_stencil: None,
            sequence: false,
            order: 0,
            viewport: ((0.0, 0.0), (1.0, 1.0)),
        }
    }
}

impl_handle!(SurfaceHandle);

impl SurfaceSetup {
    /// Sets the render target of this `Surface` layer. If `framebuffer` is none,
    /// default framebuffer will be used as render target.
    #[inline(always)]
    pub fn set_framebuffer<T>(&mut self, framebuffer: T)
    where
        T: Into<Option<FrameBufferHandle>>,
    {
        self.framebuffer = framebuffer.into();
    }

    /// By defaults, surface are sorted in ascending oreder by ids when rendering.
    /// For dynamic renderers where order might not be known until the last moment,
    /// surface ids can be remaped to arbitrary `order`.
    #[inline(always)]
    pub fn set_order(&mut self, order: u64) {
        self.order = order;
    }

    /// Sets the clear flags for this surface.A
    #[inline(always)]
    pub fn set_clear<C, D, S>(&mut self, color: C, depth: D, stentil: S)
    where
        C: Into<Option<Color>>,
        D: Into<Option<f32>>,
        S: Into<Option<i32>>,
    {
        self.clear_color = color.into();
        self.clear_depth = depth.into();
        self.clear_stencil = stentil.into();
    }

    /// Sets the viewport of view. This specifies the affine transformation of (x, y) from
    /// NDC(normalized device coordinates) to normalized window coordinates.
    #[inline(always)]
    pub fn set_viewport<T>(&mut self, position: (f32, f32), size: (f32, f32)) {
        self.viewport = (position, size);
    }

    /// Sets the sequence mode enable.
    ///
    /// Drawcalls inside `Surface` are sorted before submitting to underlaying OpenGL as
    /// default. In case where order has to be preserved (for example in rendering GUIs),
    /// `Surface` can be set to be in sequential order.
    ///
    /// Sequential order is less efficient, because it doesn't allow state change
    /// optimization, and should be avoided when possible.
    #[inline(always)]
    pub fn set_sequence(&mut self, sequence: bool) {
        self.sequence = sequence;
    }
}

/// `FrameBuffer` is a collection of 2D arrays or storages, including
/// color buffers, depth buffer, stencil buffer.
#[derive(Debug, Default, Copy, Clone)]
pub struct FrameBufferSetup {
    attachments: [Option<FrameBufferAttachment>; MAX_FRAMEBUFFER_ATTACHMENTS],
}

#[derive(Debug, Clone, Copy)]
pub enum FrameBufferAttachment {
    Texture(RenderTextureHandle),
    RenderBuffer(RenderBufferHandle),
}

impl Into<FrameBufferAttachment> for RenderTextureHandle {
    fn into(self) -> FrameBufferAttachment {
        FrameBufferAttachment::Texture(self)
    }
}

impl Into<FrameBufferAttachment> for RenderBufferHandle {
    fn into(self) -> FrameBufferAttachment {
        FrameBufferAttachment::RenderBuffer(self)
    }
}

impl FrameBufferSetup {
    /// Gets the underlying attachments.
    #[inline(always)]
    pub fn attachments(&self) -> &[Option<FrameBufferAttachment>] {
        &self.attachments[..]
    }

    /// Attach a `RenderBufferObject` or `TextureObject` as a logical buffer to the
    /// `FrameBufferObject`.
    #[inline(always)]
    pub fn set_attachment<T, S>(&mut self, handle: T, slot: S) -> Result<()>
    where
        T: Into<FrameBufferAttachment>,
        S: Into<Option<usize>>,
    {
        let slot = slot.into().unwrap_or(0);
        if slot >= MAX_FRAMEBUFFER_ATTACHMENTS {
            bail!("out of bounds");
        }

        self.attachments[slot] = Some(handle.into());
        Ok(())
    }
}

impl_handle!(FrameBufferHandle);

/// Defines a rectangle, called the scissor box, in window coordinates. The test is
/// initially disabled. While the test is enabled, only pixels that lie within the
/// scissor box can be modified by drawing commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scissor {
    Enable((u16, u16), (u16, u16)),
    Disable,
}
