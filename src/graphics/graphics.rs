use std::ops::Deref;
use std::sync::{Arc, RwLock, Mutex, MutexGuard};
use glutin;
use utility::HandleObjectSet;

use super::*;
use super::errors::*;
use super::frame::*;
use super::resource::*;
use super::pipeline::*;
use super::color::Color;
use super::backend::Context;

pub struct Graphics {
    context: Context,

    views: HandleObjectSet<Arc<RwLock<ViewStateObject>>>,
    pipelines: HandleObjectSet<Arc<RwLock<PipelineStateObject>>>,
    vertex_buffers: HandleObjectSet<Arc<RwLock<VertexBufferObject>>>,
    index_buffers: HandleObjectSet<Arc<RwLock<IndexBufferObject>>>,
    textures: HandleObjectSet<Arc<RwLock<TextureObject>>>,
    renderbuffers: HandleObjectSet<Arc<RwLock<RenderBufferObject>>>,
    framebuffers: HandleObjectSet<Arc<RwLock<FrameBufferObject>>>,
    handle_buf: Vec<Handle>,

    frames: DoubleFrame,
    multithread: bool,
}

impl Graphics {
    /// Create a new `Graphics` with `glutin::Window`.
    pub fn new(window: Arc<glutin::Window>) -> Result<Self> {
        Ok(Graphics {
               context: Context::new(window)?,
               views: HandleObjectSet::new(),
               pipelines: HandleObjectSet::new(),
               vertex_buffers: HandleObjectSet::new(),
               index_buffers: HandleObjectSet::new(),
               textures: HandleObjectSet::new(),
               renderbuffers: HandleObjectSet::new(),
               framebuffers: HandleObjectSet::new(),
               handle_buf: Vec::new(),
               frames: DoubleFrame::with_capacity(64 * 1024), // 64 kbs
               multithread: false,
           })
    }

    /// Advance to next frame. When using multithreaded renderer, this call just swaps internal
    /// buffers, kick render thread, and returns. In single threaded renderer this call does
    /// blocking frame rendering.
    pub fn run_one_frame(&mut self) -> Result<()> {
        let mut frame = self.frames.front();

        self.handle_buf.clear();
        // Update view object parameters or free vso if neccessary.
        for handle in self.views.iter() {
            let item = self.views.get(handle).unwrap();
            if Arc::strong_count(&item) == 1 {
                self.handle_buf.push(handle);
            } else {
                let mut vso = item.write().unwrap();
                let handle = handle.into();

                // Update view's render target.
                if let Some(framebuffer) = vso.update_framebuffer {
                    frame
                        .pre
                        .push(PreFrameTask::UpdateViewFrameBuffer(handle, framebuffer));
                    vso.update_framebuffer = None;
                }

                // Update view's draw order.
                if let Some(order) = vso.update_order {
                    frame.pre.push(PreFrameTask::UpdateViewOrder(handle, order));
                    vso.update_order = None;
                }

                // Update view's sequential mode.
                if let Some(seq) = vso.update_seq_mode {
                    frame
                        .pre
                        .push(PreFrameTask::UpdateViewSequential(handle, seq));
                    vso.update_seq_mode = None;
                }

                // Update view's viewport.
                if let Some(viewport) = vso.update_viewport {
                    let ptr = frame
                        .buf
                        .extend(&ViewRectDesc {
                                    position: viewport.0,
                                    size: viewport.1,
                                });
                    frame.pre.push(PreFrameTask::UpdateViewRect(handle, ptr));
                    vso.update_viewport = None;
                }
            }
        }

        for handle in self.handle_buf.drain(..) {
            frame.post.push(PostFrameTask::DeleteView(handle.into()));
            self.views.free(handle);
        }

        // Update pipeline state parameters or free pso if neccessary.
        for handle in self.pipelines.iter() {
            let item = self.pipelines.get(handle).unwrap();
            if Arc::strong_count(&item) == 1 {
                self.handle_buf.push(handle);
            } else {
                let mut pso = item.write().unwrap();
                let handle = handle.into();

                // Update pipeline's render state.
                if let Some(state) = pso.update_state {
                    let ptr = frame.buf.extend(&state);
                    frame
                        .pre
                        .push(PreFrameTask::UpdatePipelineState(handle, ptr));
                    pso.update_state = None;
                }
            }
        }

        for handle in self.handle_buf.drain(..) {
            frame
                .post
                .push(PostFrameTask::DeletePipeline(handle.into()));
            self.pipelines.free(handle);
        }

        // Update framebuffer parameters or free framebuffer object if neccessary.
        for handle in self.framebuffers.iter() {
            let item = self.framebuffers.get(handle).unwrap();
            // If this framebuffer is owned by `Graphics` only, then free it.
            if Arc::strong_count(&item) == 1 {
                self.handle_buf.push(handle);
            } else {
                let mut fbo = item.write().unwrap();
                let handle = handle.into();

                // Update framebuffer's clear option.
                if let Some(clear) = fbo.update_clear {
                    let ptr = frame
                        .buf
                        .extend(&FrameBufferClearDesc {
                                    clear_color: clear.0.map(|v| v.into()),
                                    clear_depth: clear.1,
                                    clear_stencil: clear.2,
                                });
                    frame
                        .pre
                        .push(PreFrameTask::UpdateFrameBufferClear(handle, ptr));
                    fbo.update_clear = None;
                }

                // Update framebuffer's attachments.
                for i in 0..MAX_ATTACHMENTS {
                    if let Some(v) = fbo.update_attachments[i] {
                        let i = i as u32;
                        frame
                            .pre
                            .push(PreFrameTask::UpdateFrameBufferAttachment(handle, i, v));
                        fbo.update_attachments[i as usize] = None;
                    }
                }
            }
        }

        for handle in self.handle_buf.drain(..) {
            frame
                .post
                .push(PostFrameTask::DeleteFrameBuffer(handle.into()));
            self.framebuffers.free(handle);
        }

        // Update texture parameters or free video texture object if necessary.
        for handle in self.textures.iter() {
            let item = self.textures.get(handle).unwrap();
            if Arc::strong_count(&item) == 1 {
                self.handle_buf.push(handle);
            } else {
                let mut texture = item.write().unwrap();
                if let Some((address, filter)) = texture.update_params {
                    let ptr = frame
                        .buf
                        .extend(&TextureParametersDesc {
                                    address: address,
                                    filter: filter,
                                });
                    frame
                        .pre
                        .push(PreFrameTask::UpdateTextureParameters(handle.into(), ptr));
                    texture.update_params = None;
                }
            }
        }

        for handle in self.handle_buf.drain(..) {
            frame.post.push(PostFrameTask::DeleteTexture(handle.into()));
            self.textures.free(handle);
        }

        // Free render buffer object if necessary.
        for handle in self.renderbuffers.iter() {
            let item = self.renderbuffers.get(handle).unwrap();
            if Arc::strong_count(&item) == 1 {
                self.handle_buf.push(handle);
            }
        }

        for handle in self.handle_buf.drain(..) {
            frame
                .post
                .push(PostFrameTask::DeleteRenderBuffer(handle.into()));
            self.renderbuffers.free(handle);
        }

        // Free vertex buffer object if necessary.
        for handle in self.vertex_buffers.iter() {
            let item = self.vertex_buffers.get(handle).unwrap();
            if Arc::strong_count(&item) == 1 {
                self.handle_buf.push(handle);
            }
        }

        for handle in self.handle_buf.drain(..) {
            frame
                .post
                .push(PostFrameTask::DeleteVertexBuffer(handle.into()));
            self.vertex_buffers.free(handle);
        }

        // Free index buffer object if necessary.
        for handle in self.index_buffers.iter() {
            let item = self.index_buffers.get(handle).unwrap();
            if Arc::strong_count(&item) == 1 {
                self.handle_buf.push(handle);
            }
        }

        for handle in self.handle_buf.drain(..) {
            frame
                .post
                .push(PostFrameTask::DeleteIndexBuffer(handle.into()));
            self.index_buffers.free(handle);
        }

        unsafe {
            if !self.multithread {
                self.context.device().run_one_frame()?;
                self.frames.swap_frames();
                self.frames.back().dispatch(&mut self.context)?;
                self.frames.back().clear();
                self.context.swap_buffers()?;
            }

            Ok(())
        }
    }
}

/// View represent bucket of draw calls. Drawcalls inside bucket are sorted before
/// submitting to underlaying OpenGL. In case where order has to be preserved (for
/// example in rendering GUIs), view can be set to be in sequential order. Sequential
/// order is less efficient, because it doesn't allow state change optimization, and
/// should be avoided when possible.
#[derive(Debug)]
pub struct ViewStateObject {
    framebuffer: Option<FrameBufferRef>,
    update_framebuffer: Option<Option<FrameBufferHandle>>,
    update_order: Option<u32>,
    update_seq_mode: Option<bool>,
    update_viewport: Option<((u16, u16), Option<(u16, u16)>)>,
}

impl ViewStateObject {
    /// Update the render target of `View` bucket. If `framebuffer` is none, default
    /// framebuffer will be used as render target
    #[inline]
    pub fn update_framebuffer(&mut self, framebuffer: Option<&FrameBufferRef>) {
        self.framebuffer = framebuffer.map(|v| v.clone());
        self.update_framebuffer = Some(framebuffer.map(|v| v.handle));
    }

    /// By defaults view are sorted in ascending oreder by ids when rendering.
    /// For dynamic renderers where order might not be known until the last moment,
    /// view ids can be remaped to arbitrary order by calling `update_order`.
    #[inline]
    pub fn update_order(&mut self, order: u32) {
        self.update_order = Some(order);
    }

    /// Set view into sequential mode. Drawcalls will be sorted in the same order in which submit calls
    /// were called.
    #[inline]
    pub fn update_sequential_mode(&mut self, seq: bool) {
        self.update_seq_mode = Some(seq);
    }

    /// Set the viewport of view. This specifies the affine transformation of (x, y) from
    /// NDC(normalized device coordinates) to window coordinates.
    ///
    /// If `size` is none, the dimensions of framebuffer will be used as size
    #[inline]
    pub fn update_viewport(&mut self, position: (u16, u16), size: Option<(u16, u16)>) {
        self.update_viewport = Some((position, size));
    }
}

#[derive(Debug, Clone)]
pub struct ViewStateRef {
    pub handle: ViewHandle,
    pub object: Arc<RwLock<ViewStateObject>>,
}

impl Deref for ViewStateRef {
    type Target = ViewHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Graphics {
    /// Creates an view with optional `FrameBuffer`. If `FrameBuffer` is none, default
    /// framebuffer will be used as render target.
    pub fn create_view(&mut self, framebuffer: Option<&FrameBufferRef>) -> Result<ViewStateRef> {
        let object = Arc::new(RwLock::new(ViewStateObject {
                                              framebuffer: framebuffer.map(|v| v.clone()),
                                              update_framebuffer: None,
                                              update_order: None,
                                              update_seq_mode: None,
                                              update_viewport: None,
                                          }));

        let mut frame = self.frames.front();
        let handle = self.views.create(object.clone()).into();
        frame
            .pre
            .push(PreFrameTask::CreateView(handle, framebuffer.map(|v| v.handle)));

        Ok(ViewStateRef {
               handle: handle,
               object: object,
           })
    }
}

#[derive(Debug)]
pub struct PipelineStateObject {
    attributes: AttributeLayout,
    update_state: Option<RenderState>,
}

impl PipelineStateObject {
    #[inline]
    pub fn attributes(&self) -> &AttributeLayout {
        &self.attributes
    }

    #[inline]
    pub fn update_state(&mut self, state: &RenderState) {
        self.update_state = Some(*state);
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStateRef {
    pub handle: PipelineStateHandle,
    pub object: Arc<RwLock<PipelineStateObject>>,
}

impl Deref for PipelineStateRef {
    type Target = PipelineStateHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Graphics {
    /// Create a pipeline with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_pipeline(&mut self,
                           vs: &str,
                           fs: &str,
                           state: &RenderState,
                           attributes: &AttributeLayout)
                           -> Result<PipelineStateRef> {
        let object = Arc::new(RwLock::new(PipelineStateObject {
                                              attributes: *attributes,
                                              update_state: None,
                                          }));

        let mut frame = self.frames.front();
        let handle = self.pipelines.create(object.clone()).into();

        let vs = frame.buf.extend_from_str(vs);
        let fs = frame.buf.extend_from_str(fs);

        let ptr = frame
            .buf
            .extend(&PipelineDesc {
                        vs: vs,
                        fs: fs,
                        state: *state,
                        attributes: *attributes,
                    });

        frame.pre.push(PreFrameTask::CreatePipeline(handle, ptr));
        Ok(PipelineStateRef {
               handle: handle,
               object: object,
           })
    }
}

#[derive(Debug)]
pub struct FrameBufferObject {
    renderbuffers: [Option<RenderBufferRef>; MAX_ATTACHMENTS],
    textures: [Option<TextureRef>; MAX_ATTACHMENTS],
    update_clear: Option<(Option<Color>, Option<f32>, Option<i32>)>,
    update_attachments: [Option<FrameBufferAttachment>; MAX_ATTACHMENTS],
}

impl FrameBufferObject {
    /// Update the clear color of `FrameBufferObject`.
    #[inline]
    pub fn update_clear(&mut self,
                        color: Option<Color>,
                        depth: Option<f32>,
                        stencil: Option<i32>) {
        self.update_clear = Some((color, depth, stencil));
    }

    /// Attach a `RenderBufferObject` as a logical buffer to the `FrameBufferObject`.
    ///
    /// `FrameBufferObject` will keep a reference to this attachment, so its perfectly
    /// safe to drop attached resource immediately.
    pub fn update_attachment(&mut self,
                             attachment: &RenderBufferRef,
                             slot: Option<usize>)
                             -> Result<()> {
        let handle = FrameBufferAttachment::RenderBuffer(attachment.handle);
        let slot = match attachment.object.read().unwrap().format() {
            RenderTextureFormat::RGB8 |
            RenderTextureFormat::RGBA4 |
            RenderTextureFormat::RGBA8 => {
                let slot = slot.unwrap_or(0);
                if slot >= MAX_ATTACHMENTS - 1 {
                    bail!("out of bounds.");
                }
                slot
            }
            RenderTextureFormat::Depth16 |
            RenderTextureFormat::Depth24 |
            RenderTextureFormat::Depth32 |
            RenderTextureFormat::Depth24Stencil8 => MAX_ATTACHMENTS - 1,
        };

        self.update_attachments[slot] = Some(handle);
        self.renderbuffers[slot] = Some(attachment.clone());
        Ok(())
    }

    /// Attach a `TextureObject` as a logical buffer to the `FrameBufferObject`.
    ///
    /// `FrameBufferObject` will keep a reference to this attachment, so its perfectly
    /// safe to drop attached resource immediately.
    pub fn update_texture_attachment(&mut self,
                                     attachment: &TextureRef,
                                     slot: Option<usize>)
                                     -> Result<()> {
        let handle = FrameBufferAttachment::Texture(attachment.handle);
        let slot = match attachment.object.read().unwrap().render_format() {
            RenderTextureFormat::RGB8 |
            RenderTextureFormat::RGBA4 |
            RenderTextureFormat::RGBA8 => {
                let slot = slot.unwrap_or(0);
                if slot >= MAX_ATTACHMENTS - 1 {
                    bail!("out of bounds.");
                }
                slot
            }
            RenderTextureFormat::Depth16 |
            RenderTextureFormat::Depth24 |
            RenderTextureFormat::Depth32 |
            RenderTextureFormat::Depth24Stencil8 => MAX_ATTACHMENTS - 1,
        };

        self.update_attachments[slot] = Some(handle);
        self.textures[slot] = Some(attachment.clone());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FrameBufferRef {
    pub handle: FrameBufferHandle,
    pub object: Arc<RwLock<FrameBufferObject>>,
}

impl Deref for FrameBufferRef {
    type Target = FrameBufferHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Graphics {
    /// Create a framebuffer object. A framebuffer allows you to render primitives directly to a texture,
    /// which can then be used in other rendering operations.
    ///
    /// At least one color attachment has been attached before you can use it.
    pub fn create_framebuffer(&mut self) -> Result<FrameBufferRef> {
        let object = Arc::new(RwLock::new(FrameBufferObject {
                                              renderbuffers: [None, None, None, None, None, None,
                                                              None, None],
                                              textures: [None, None, None, None, None, None, None,
                                                         None],
                                              update_clear: None,
                                              update_attachments: [None; MAX_ATTACHMENTS],
                                          }));

        let handle = self.framebuffers.create(object.clone()).into();
        let mut frame = self.frames.front();
        frame.pre.push(PreFrameTask::CreateFrameBuffer(handle));

        Ok(FrameBufferRef {
               handle: handle,
               object: object,
           })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextureObject {
    format: TextureFormat,
    render_format: RenderTextureFormat,
    dimensions: (u32, u32),
    update_params: Option<(TextureAddress, TextureFilter)>,
}

impl TextureObject {
    #[inline]
    pub fn render_format(&self) -> RenderTextureFormat {
        self.render_format
    }

    #[inline]
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    #[inline]
    pub fn update_parameters(&mut self, address: TextureAddress, filter: TextureFilter) {
        self.update_params = Some((address, filter));
    }
}

#[derive(Debug, Clone)]
pub struct TextureRef {
    pub handle: TextureHandle,
    pub object: Arc<RwLock<TextureObject>>,
}

impl Deref for TextureRef {
    type Target = TextureHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Graphics {
    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture(&mut self,
                          format: TextureFormat,
                          address: TextureAddress,
                          filter: TextureFilter,
                          mipmap: bool,
                          width: u32,
                          height: u32,
                          data: &[u8])
                          -> Result<TextureRef> {
        let object = Arc::new(RwLock::new(TextureObject {
                                              format: format,
                                              render_format: RenderTextureFormat::RGBA8,
                                              dimensions: (width, height),
                                              update_params: None,
                                          }));

        let mut frame = self.frames.front();
        let handle = self.textures.create(object.clone()).into();

        let data = frame.buf.extend_from_slice(data);
        let ptr = frame
            .buf
            .extend(&TextureDesc {
                        format: format,
                        address: address,
                        filter: filter,
                        mipmap: mipmap,
                        width: width,
                        height: height,
                        data: data,
                    });

        frame.pre.push(PreFrameTask::CreateTexture(handle, ptr));
        Ok(TextureRef {
               handle: handle,
               object: object,
           })
    }

    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(&mut self,
                                 format: RenderTextureFormat,
                                 width: u32,
                                 height: u32)
                                 -> Result<TextureRef> {
        let object = Arc::new(RwLock::new(TextureObject {
                                              format: TextureFormat::U8,
                                              render_format: format,
                                              dimensions: (width, height),
                                              update_params: None,
                                          }));

        let mut frame = self.frames.front();
        let handle = self.textures.create(object.clone()).into();

        let ptr = frame
            .buf
            .extend(&RenderTextureDesc {
                        format: format,
                        width: width,
                        height: height,
                    });

        frame
            .pre
            .push(PreFrameTask::CreateRenderTexture(handle, ptr));
        Ok(TextureRef {
               handle: handle,
               object: object,
           })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RenderBufferObject {
    format: RenderTextureFormat,
    dimensions: (u32, u32),
}

impl RenderBufferObject {
    #[inline]
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    #[inline]
    pub fn format(&self) -> RenderTextureFormat {
        self.format
    }
}

#[derive(Debug, Clone)]
pub struct RenderBufferRef {
    pub handle: RenderBufferHandle,
    pub object: Arc<RwLock<RenderBufferObject>>,
}

impl Deref for RenderBufferRef {
    type Target = RenderBufferHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Graphics {
    /// Create a render buffer object, which could be attached to framebuffer.
    pub fn create_render_buffer(&mut self,
                                format: RenderTextureFormat,
                                width: u32,
                                height: u32)
                                -> Result<RenderBufferRef> {
        let object = Arc::new(RwLock::new(RenderBufferObject {
                                              format: format,
                                              dimensions: (width, height),
                                          }));

        let mut frame = self.frames.front();
        let handle = self.renderbuffers.create(object.clone()).into();

        let ptr = frame
            .buf
            .extend(&RenderTextureDesc {
                        format: format,
                        width: width,
                        height: height,
                    });

        frame
            .pre
            .push(PreFrameTask::CreateRenderBuffer(handle, ptr));
        Ok(RenderBufferRef {
               handle: handle,
               object: object,
           })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct VertexBufferObject {
    hint: ResourceHint,
    len: u32,
    layout: VertexLayout,
}

impl VertexBufferObject {
    #[inline]
    pub fn hint(&self) -> ResourceHint {
        self.hint
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    pub fn layout(&self) -> &VertexLayout {
        &self.layout
    }
}

#[derive(Debug, Clone)]
pub struct VertexBufferRef {
    pub handle: VertexBufferHandle,
    pub object: Arc<RwLock<VertexBufferObject>>,
}

impl Deref for VertexBufferRef {
    type Target = VertexBufferHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Graphics {
    /// Create vertex buffer object with vertex layout declaration and optional data.
    pub fn create_vertex_buffer(&mut self,
                                layout: &VertexLayout,
                                hint: ResourceHint,
                                size: u32,
                                data: Option<&[u8]>)
                                -> Result<VertexBufferRef> {
        let object = Arc::new(RwLock::new(VertexBufferObject {
                                              hint: hint,
                                              len: size,
                                              layout: *layout,
                                          }));

        let mut frame = self.frames.front();
        let handle = self.vertex_buffers.create(object.clone()).into();

        let data = data.map(|v| frame.buf.extend_from_slice(v));
        let ptr = frame
            .buf
            .extend(&VertexBufferDesc {
                        layout: *layout,
                        hint: hint,
                        size: size,
                        data: data,
                    });

        frame
            .pre
            .push(PreFrameTask::CreateVertexBuffer(handle, ptr));
        Ok(VertexBufferRef {
               handle: handle,
               object: object,
           })
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(&mut self,
                                handle: VertexBufferHandle,
                                offset: u32,
                                data: &[u8])
                                -> Result<()> {
        if let Some(vbo) = self.vertex_buffers.get(handle) {
            let vbo = vbo.read().unwrap();
            if vbo.hint == ResourceHint::Static {
                bail!("failed to update static vertex buffer.");
            }

            if vbo.len < offset + data.len() as u32 {
                bail!("out of bounds.");
            }

            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            frame
                .pre
                .push(PreFrameTask::UpdateVertexBuffer(handle, offset, ptr));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IndexBufferObject {
    hint: ResourceHint,
    len: u32,
    format: IndexFormat,
}

impl IndexBufferObject {
    #[inline]
    pub fn hint(&self) -> ResourceHint {
        self.hint
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    pub fn format(&self) -> IndexFormat {
        self.format
    }
}

#[derive(Debug, Clone)]
pub struct IndexBufferRef {
    pub handle: IndexBufferHandle,
    pub object: Arc<RwLock<IndexBufferObject>>,
}

impl Deref for IndexBufferRef {
    type Target = IndexBufferHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Graphics {
    /// Create index buffer object with optional data.
    pub fn create_index_buffer(&mut self,
                               format: IndexFormat,
                               hint: ResourceHint,
                               size: u32,
                               data: Option<&[u8]>)
                               -> Result<IndexBufferRef> {
        let object = Arc::new(RwLock::new(IndexBufferObject {
                                              hint: hint,
                                              len: size,
                                              format: format,
                                          }));

        let mut frame = self.frames.front();
        let handle = self.index_buffers.create(object.clone()).into();

        let data = data.map(|v| frame.buf.extend_from_slice(v));
        let ptr = frame
            .buf
            .extend(&IndexBufferDesc {
                        format: format,
                        hint: hint,
                        size: size,
                        data: data,
                    });

        frame.pre.push(PreFrameTask::CreateIndexBuffer(handle, ptr));
        Ok(IndexBufferRef {
               handle: handle,
               object: object,
           })
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_index_buffer(&mut self,
                               handle: IndexBufferHandle,
                               offset: u32,
                               data: &[u8])
                               -> Result<()> {
        if let Some(ibo) = self.index_buffers.get(handle) {
            let ibo = ibo.read().unwrap();
            if ibo.hint == ResourceHint::Static {
                bail!("failed to update static vertex buffer.");
            }

            if ibo.len < offset + data.len() as u32 {
                bail!("out of bounds.");
            }

            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            frame
                .pre
                .push(PreFrameTask::UpdateIndexBuffer(handle, offset, ptr));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }
}

impl Graphics {
    /// Submit primitive for drawing, within view all draw commands are executed after
    /// resource manipulation, such like `create_vertex_buffer`, `update_vertex_buffer`, etc.
    pub fn draw(&mut self,
                priority: u64,
                view: ViewHandle,
                pipeline: PipelineStateHandle,
                textures: &[(&str, TextureHandle)],
                uniforms: &[(&str, UniformVariable)],
                vb: VertexBufferHandle,
                ib: Option<IndexBufferHandle>,
                primitive: Primitive,
                from: u32,
                len: u32)
                -> Result<()> {
        let mut frame = self.frames.front();

        let uniforms = {
            let mut variables = vec![];
            for &(name, variable) in uniforms {
                variables.push((frame.buf.extend_from_str(name), variable));
            }
            frame.buf.extend_from_slice(variables.as_slice())
        };

        let textures = {
            let mut variables = vec![];
            for &(name, variable) in textures {
                variables.push((frame.buf.extend_from_str(name), variable));
            }
            frame.buf.extend_from_slice(variables.as_slice())
        };

        frame
            .drawcalls
            .push(FrameTask {
                      priority: priority,
                      view: view,
                      pipeline: pipeline,
                      primitive: primitive,
                      vb: vb,
                      ib: ib,
                      from: from,
                      len: len,
                      textures: textures,
                      uniforms: uniforms,
                  });

        Ok(())
    }
}

struct DoubleFrame {
    idx: RwLock<usize>,
    frames: [Mutex<Frame>; 2],
}

impl DoubleFrame {
    pub fn with_capacity(capacity: usize) -> Self {
        DoubleFrame {
            idx: RwLock::new(0),
            frames: [Mutex::new(Frame::with_capacity(capacity)),
                     Mutex::new(Frame::with_capacity(capacity))],
        }
    }

    #[inline]
    pub fn front(&self) -> MutexGuard<Frame> {
        self.frames[*self.idx.read().unwrap()].lock().unwrap()
    }

    #[inline]
    pub fn back(&self) -> MutexGuard<Frame> {
        self.frames[*self.idx.read().unwrap()].lock().unwrap()
    }

    #[inline]
    pub fn swap_frames(&self) {
        let mut idx = self.idx.write().unwrap();
        *idx = (*idx + 1) % 2;
    }
}