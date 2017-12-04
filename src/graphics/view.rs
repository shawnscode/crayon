//! Named bucket of draw calls with the wrapping of rendering operations to a render
//! target, clearing, MSAA resolving and so on.

use utils::Color;
use super::errors::*;
use super::texture::{TextureHandle, RenderBufferHandle};

pub const MAX_ATTACHMENTS: usize = 8;

/// View represent bucket of draw calls. Drawcalls inside bucket are sorted before
/// submitting to underlaying OpenGL. In case where order has to be preserved (for
/// example in rendering GUIs), view can be set to be in sequential order. Sequential
/// order is less efficient, because it doesn't allow state change optimization, and
/// should be avoided when possible.
#[derive(Debug, Copy, Clone)]
pub struct ViewStateSetup {
    /// The render target of `View` bucket. If `framebuffer` is none, default
    /// framebuffer will be used as render target.
    pub framebuffer: Option<FrameBufferHandle>,
    /// The clear color.
    pub clear_color: Option<Color>,
    /// The clear depth.
    pub clear_depth: Option<f32>,
    /// The clear stencil.
    pub clear_stencil: Option<i32>,
    /// By defaults view are sorted in ascending oreder by ids when rendering.
    /// For dynamic renderers where order might not be known until the last moment,
    /// view ids can be remaped to arbitrary order.
    pub order: u32,
    /// Set view into sequential mode. Drawcalls will be sorted in the same order
    /// in which submit calls were called.
    pub sequence: bool,
    /// Set the viewport of view. This specifies the affine transformation of (x, y) from
    /// NDC(normalized device coordinates) to window coordinates.
    ///
    /// If `size` is none, the dimensions of framebuffer will be used as size
    pub viewport: ((u16, u16), Option<(u16, u16)>),
}

impl Default for ViewStateSetup {
    fn default() -> Self {
        ViewStateSetup {
            framebuffer: None,
            clear_color: Some(Color::black()),
            clear_depth: Some(1.0),
            clear_stencil: None,
            order: 0,
            sequence: false,
            viewport: ((0, 0), None),
        }
    }
}

impl_handle!(ViewStateHandle);

/// `FrameBuffer` is a collection of 2D arrays or storages, including
/// color buffers, depth buffer, stencil buffer.
#[derive(Debug, Default, Copy, Clone)]
pub struct FrameBufferSetup {
    attachments: [Option<FrameBufferAttachment>; MAX_ATTACHMENTS],
}

#[derive(Debug, Clone, Copy)]
pub enum FrameBufferAttachment {
    Texture(TextureHandle),
    RenderBuffer(RenderBufferHandle),
}

impl FrameBufferSetup {
    pub fn attachments(&self) -> &[Option<FrameBufferAttachment>] {
        &self.attachments[..]
    }

    /// Attach a `RenderBufferObject` as a logical buffer to the `FrameBufferObject`.
    pub fn set_attachment(&mut self,
                          handle: RenderBufferHandle,
                          slot: Option<usize>)
                          -> Result<()> {
        let slot = slot.unwrap_or(0);
        if slot >= MAX_ATTACHMENTS {
            bail!("out of bounds");
        }

        self.attachments[slot] = Some(FrameBufferAttachment::RenderBuffer(handle));
        Ok(())
    }

    /// Attach a `TextureObject` as a logical buffer to the `FrameBufferObject`.
    pub fn set_texture_attachment(&mut self,
                                  handle: TextureHandle,
                                  slot: Option<usize>)
                                  -> Result<()> {
        let slot = slot.unwrap_or(0);
        if slot >= MAX_ATTACHMENTS {
            bail!("out of bounds");
        }

        self.attachments[slot] = Some(FrameBufferAttachment::Texture(handle));
        Ok(())
    }
}

impl_handle!(FrameBufferHandle);