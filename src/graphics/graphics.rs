//! Public interface of graphics module.

use std::sync::{Arc, RwLock, Mutex, MutexGuard};
use std::time::Duration;

use utils::HandlePool;

use super::*;
use super::errors::*;
use super::frame::*;
use super::backend::Device;
use super::window::Window;

#[derive(Debug, Copy, Clone, Default)]
pub struct GraphicsFrameInfo {
    pub duration: Duration,
    pub drawcall: usize,
    pub alive_views: usize,
    pub alive_pipelines: usize,
    pub alive_frame_buffers: usize,
    pub alive_vertex_buffers: usize,
    pub alive_index_buffers: usize,
    pub alive_textures: usize,
    pub alive_render_buffers: usize,
}

pub struct GraphicsSystem {
    window: Arc<Window>,
    device: Device,
    frames: Arc<DoubleFrame>,
    shared: Arc<RwLock<GraphicsSystemShared>>,
}

impl GraphicsSystem {
    /// Create a new `GraphicsSystem` with one `Window` context.
    pub fn new(window: Arc<window::Window>) -> Result<Self> {
        let device = unsafe { Device::new() };
        let frames = Arc::new(DoubleFrame::with_capacity(64 * 1024));

        let shared = GraphicsSystemShared::new(frames.clone());

        Ok(GraphicsSystem {
               window: window,
               device: device,
               frames: frames,
               shared: Arc::new(RwLock::new(shared)),
           })
    }

    pub fn shared(&self) -> Arc<RwLock<GraphicsSystemShared>> {
        self.shared.clone()
    }

    #[inline]
    pub fn swap_frames(&self) {
        self.frames.swap_frames();
    }

    /// Advance to next frame.
    ///
    /// Notes that this method MUST be called at main thread, and will NOT return
    /// until all commands is finished by GPU.
    pub fn advance(&mut self) -> Result<GraphicsFrameInfo> {
        use std::time;
        let mut info = GraphicsFrameInfo::default();

        unsafe {
            let ts = time::Instant::now();
            let dimensions = self.window.dimensions().ok_or(ErrorKind::WindowNotExist)?;

            {
                self.device.run_one_frame()?;

                {
                    let mut frame = self.frames.back();
                    info.drawcall = frame.drawcalls.len();
                    frame.dispatch(&mut self.device, dimensions)?;
                    frame.clear();
                }
            }

            self.window.swap_buffers()?;

            info.duration = time::Instant::now() - ts;

            let shared = self.shared.read().unwrap();
            info.alive_views = shared.views.size();
            info.alive_pipelines = shared.pipelines.size();
            info.alive_frame_buffers = shared.framebuffers.size();
            info.alive_vertex_buffers = shared.vertex_buffers.size();
            info.alive_index_buffers = shared.index_buffers.size();
            info.alive_textures = shared.textures.size();
            info.alive_render_buffers = shared.render_buffers.size();

            Ok(info)
        }
    }
}

/// The frontend of graphics module.
pub struct GraphicsSystemShared {
    views: HandlePool,
    pipelines: HandlePool,
    framebuffers: HandlePool,
    vertex_buffers: HandlePool,
    index_buffers: HandlePool,
    textures: HandlePool,
    render_buffers: HandlePool,
    frames: Arc<DoubleFrame>,
}

impl GraphicsSystemShared {
    /// Create a new `GraphicsSystem` with one `Window` context.
    fn new(frames: Arc<DoubleFrame>) -> Self {
        GraphicsSystemShared {
            frames: frames,

            views: HandlePool::new(),
            framebuffers: HandlePool::new(),
            pipelines: HandlePool::new(),
            vertex_buffers: HandlePool::new(),
            index_buffers: HandlePool::new(),
            textures: HandlePool::new(),
            render_buffers: HandlePool::new(),
        }
    }

    /// Make a new draw call.
    #[inline]
    pub fn make(&self) -> DrawCallBuilder {
        DrawCallBuilder::new(self.frames.front())
    }
}

impl GraphicsSystemShared {
    /// Creates an view with `ViewStateSetup`.
    pub fn create_view(&mut self, setup: ViewStateSetup) -> Result<ViewStateHandle> {
        let mut frame = self.frames.front();
        let handle = self.views.create().into();
        frame.pre.push(PreFrameTask::CreateView(handle, setup));
        Ok(handle)
    }

    /// Deletes a view state object.
    pub fn delete_view(&mut self, handle: ViewStateHandle) {
        if self.views.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteView(handle));
        }
    }
}

impl GraphicsSystemShared {
    /// Create a pipeline with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_pipeline(&mut self,
                           setup: PipelineStateSetup,
                           vs: String,
                           fs: String)
                           -> Result<PipelineStateHandle> {
        let mut frame = self.frames.front();
        let handle = self.pipelines.create().into();
        frame
            .pre
            .push(PreFrameTask::CreatePipeline(handle, setup, vs, fs));

        Ok(handle)
    }

    /// Deletes a pipeline state object.
    pub fn delete_pipeline(&mut self, handle: PipelineStateHandle) {
        if self.pipelines.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeletePipeline(handle));
        }
    }
}

impl GraphicsSystemShared {
    /// Create a framebuffer object. A framebuffer allows you to render primitives directly to a texture,
    /// which can then be used in other rendering operations.
    ///
    /// At least one color attachment has been attached before you can use it.
    pub fn create_framebuffer(&mut self, setup: FrameBufferSetup) -> Result<FrameBufferHandle> {
        let handle = self.framebuffers.create().into();
        let mut frame = self.frames.front();
        frame
            .pre
            .push(PreFrameTask::CreateFrameBuffer(handle, setup));
        Ok(handle)
    }

    /// Deletes a framebuffer object.
    pub fn delete_framebuffer(&mut self, handle: FrameBufferHandle) {
        if self.framebuffers.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteFrameBuffer(handle));
        }
    }
}

impl GraphicsSystemShared {
    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture(&mut self, setup: TextureSetup, data: Vec<u8>) -> Result<TextureHandle> {
        let mut frame = self.frames.front();
        let handle = self.textures.create().into();
        frame
            .pre
            .push(PreFrameTask::CreateTexture(handle, setup, data));
        Ok(handle)
    }

    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(&mut self, setup: RenderTextureSetup) -> Result<TextureHandle> {
        let mut frame = self.frames.front();
        let handle = self.textures.create().into();
        frame
            .pre
            .push(PreFrameTask::CreateRenderTexture(handle, setup));
        Ok(handle)
    }

    /// Deletes a texture object.
    pub fn delete_texture(&mut self, handle: TextureHandle) {
        if self.textures.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteTexture(handle));
        }
    }
}

impl GraphicsSystemShared {
    /// Create a render buffer object, which could be attached to framebuffer.
    pub fn create_render_buffer(&mut self, setup: RenderBufferSetup) -> Result<RenderBufferHandle> {
        let mut frame = self.frames.front();
        let handle = self.render_buffers.create().into();
        frame
            .pre
            .push(PreFrameTask::CreateRenderBuffer(handle, setup));
        Ok(handle)
    }

    /// Deletes a render buffer object.
    pub fn delete_render_buffer(&mut self, handle: RenderBufferHandle) {
        if self.render_buffers.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteRenderBuffer(handle));
        }
    }
}

impl GraphicsSystemShared {
    /// Create vertex buffer object with vertex layout declaration and optional data.
    pub fn create_vertex_buffer(&mut self,
                                setup: VertexBufferSetup,
                                data: Option<&[u8]>)
                                -> Result<VertexBufferHandle> {
        if let Some(buf) = data.as_ref() {
            if buf.len() > setup.len() {
                bail!("out of bounds");
            }
        }

        let mut frame = self.frames.front();
        let handle = self.vertex_buffers.create().into();
        let ptr = data.map(|v| frame.buf.extend_from_slice(v));

        frame
            .pre
            .push(PreFrameTask::CreateVertexBuffer(handle, setup, ptr));
        Ok(handle)
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(&mut self,
                                handle: VertexBufferHandle,
                                offset: usize,
                                data: &[u8])
                                -> Result<()> {
        if self.vertex_buffers.is_alive(handle) {
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

    /// Deletes a vertex buffer object.
    pub fn delete_vertex_buffer(&mut self, handle: VertexBufferHandle) {
        if self.vertex_buffers.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteVertexBuffer(handle));
        }
    }
}

impl GraphicsSystemShared {
    /// Create index buffer object with optional data.
    pub fn create_index_buffer(&mut self,
                               setup: IndexBufferSetup,
                               data: Option<&[u8]>)
                               -> Result<IndexBufferHandle> {
        if let Some(buf) = data.as_ref() {
            if buf.len() > setup.len() {
                bail!("out of bounds");
            }
        }

        let mut frame = self.frames.front();
        let handle = self.index_buffers.create().into();
        let ptr = data.map(|v| frame.buf.extend_from_slice(v));

        frame
            .pre
            .push(PreFrameTask::CreateIndexBuffer(handle, setup, ptr));
        Ok(handle)
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_index_buffer(&mut self,
                               handle: IndexBufferHandle,
                               offset: usize,
                               data: &[u8])
                               -> Result<()> {
        if self.index_buffers.is_alive(handle) {
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

    /// Deletes a vertex buffer object.
    pub fn delete_index_buffer(&mut self, handle: IndexBufferHandle) {
        if self.index_buffers.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteIndexBuffer(handle));
        }
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
        self.frames[(*self.idx.read().unwrap() + 1) % 2]
            .lock()
            .unwrap()
    }

    #[inline]
    pub fn swap_frames(&self) {
        let mut idx = self.idx.write().unwrap();
        *idx = (*idx + 1) % 2;
    }
}