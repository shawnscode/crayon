//! The centralized management of video sub-system.

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

/// The centralized management of video sub-system.
pub struct GraphicsSystem {
    window: Arc<Window>,
    device: Device,
    frames: Arc<DoubleFrame>,
    shared: Arc<GraphicsSystemShared>,
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
               shared: Arc::new(shared),
           })
    }

    /// Returns the multi-thread friendly parts of `GraphicsSystem`.
    pub fn shared(&self) -> Arc<GraphicsSystemShared> {
        self.shared.clone()
    }

    /// Swap internal commands frame.
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

            info.alive_views = self.shared.views.read().unwrap().size();
            info.alive_pipelines = self.shared.pipelines.read().unwrap().size();
            info.alive_frame_buffers = self.shared.framebuffers.read().unwrap().size();
            info.alive_vertex_buffers = self.shared.vertex_buffers.read().unwrap().size();
            info.alive_index_buffers = self.shared.index_buffers.read().unwrap().size();
            info.alive_textures = self.shared.textures.read().unwrap().size();
            info.alive_render_buffers = self.shared.render_buffers.read().unwrap().size();

            Ok(info)
        }
    }
}

/// The multi-thread friendly parts of `GraphicsSystem`.
pub struct GraphicsSystemShared {
    views: RwLock<HandlePool>,
    pipelines: RwLock<HandlePool>,
    framebuffers: RwLock<HandlePool>,
    vertex_buffers: RwLock<HandlePool>,
    index_buffers: RwLock<HandlePool>,
    textures: RwLock<HandlePool>,
    render_buffers: RwLock<HandlePool>,

    frames: Arc<DoubleFrame>,
}

impl GraphicsSystemShared {
    /// Create a new `GraphicsSystem` with one `Window` context.
    fn new(frames: Arc<DoubleFrame>) -> Self {
        GraphicsSystemShared {
            frames: frames,

            views: RwLock::new(HandlePool::new()),
            framebuffers: RwLock::new(HandlePool::new()),
            pipelines: RwLock::new(HandlePool::new()),
            vertex_buffers: RwLock::new(HandlePool::new()),
            index_buffers: RwLock::new(HandlePool::new()),
            textures: RwLock::new(HandlePool::new()),
            render_buffers: RwLock::new(HandlePool::new()),
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
    pub fn create_view(&self, setup: ViewStateSetup) -> Result<ViewStateHandle> {
        let handle = self.views.write().unwrap().create().into();

        {
            let task = PreFrameTask::CreateView(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Deletes a view state object.
    pub fn delete_view(&self, handle: ViewStateHandle) {
        if self.views.write().unwrap().free(handle) {
            let task = PostFrameTask::DeleteView(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Create a pipeline with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_pipeline(&self,
                           setup: PipelineStateSetup,
                           vs: String,
                           fs: String)
                           -> Result<PipelineStateHandle> {
        let handle = self.pipelines.write().unwrap().create().into();

        {
            let task = PreFrameTask::CreatePipeline(handle, setup, vs, fs);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Deletes a pipeline state object.
    pub fn delete_pipeline(&self, handle: PipelineStateHandle) {
        if self.pipelines.write().unwrap().free(handle) {
            let task = PostFrameTask::DeletePipeline(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Create a framebuffer object. A framebuffer allows you to render primitives directly to a texture,
    /// which can then be used in other rendering operations.
    ///
    /// At least one color attachment has been attached before you can use it.
    pub fn create_framebuffer(&self, setup: FrameBufferSetup) -> Result<FrameBufferHandle> {
        let handle = self.framebuffers.write().unwrap().create().into();

        {
            let task = PreFrameTask::CreateFrameBuffer(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Deletes a framebuffer object.
    pub fn delete_framebuffer(&self, handle: FrameBufferHandle) {
        if self.framebuffers.write().unwrap().free(handle) {
            let task = PostFrameTask::DeleteFrameBuffer(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture(&self, setup: TextureSetup, data: Vec<u8>) -> Result<TextureHandle> {
        let handle = self.textures.write().unwrap().create().into();

        {
            let task = PreFrameTask::CreateTexture(handle, setup, data);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(&self, setup: RenderTextureSetup) -> Result<TextureHandle> {
        let handle = self.textures.write().unwrap().create().into();

        {
            let task = PreFrameTask::CreateRenderTexture(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Deletes a texture object.
    pub fn delete_texture(&self, handle: TextureHandle) {
        if self.textures.write().unwrap().free(handle) {
            let task = PostFrameTask::DeleteTexture(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Create a render buffer object, which could be attached to framebuffer.
    pub fn create_render_buffer(&self, setup: RenderBufferSetup) -> Result<RenderBufferHandle> {
        let handle = self.render_buffers.write().unwrap().create().into();

        {
            let task = PreFrameTask::CreateRenderBuffer(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Deletes a render buffer object.
    pub fn delete_render_buffer(&self, handle: RenderBufferHandle) {
        if self.render_buffers.write().unwrap().free(handle) {
            let task = PostFrameTask::DeleteRenderBuffer(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Create vertex buffer object with vertex layout declaration and optional data.
    pub fn create_vertex_buffer(&self,
                                setup: VertexBufferSetup,
                                data: Option<&[u8]>)
                                -> Result<VertexBufferHandle> {
        if let Some(buf) = data.as_ref() {
            if buf.len() > setup.len() {
                bail!("out of bounds");
            }
        }

        let handle = self.vertex_buffers.write().unwrap().create().into();

        {
            let mut frame = self.frames.front();
            let ptr = data.map(|v| frame.buf.extend_from_slice(v));
            let task = PreFrameTask::CreateVertexBuffer(handle, setup, ptr);
            frame.pre.push(task);
        }

        Ok(handle)
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(&self,
                                handle: VertexBufferHandle,
                                offset: usize,
                                data: &[u8])
                                -> Result<()> {
        if self.vertex_buffers.read().unwrap().is_alive(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            let task = PreFrameTask::UpdateVertexBuffer(handle, offset, ptr);
            frame.pre.push(task);
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Deletes a vertex buffer object.
    pub fn delete_vertex_buffer(&self, handle: VertexBufferHandle) {
        if self.vertex_buffers.write().unwrap().free(handle) {
            let task = PostFrameTask::DeleteVertexBuffer(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Create index buffer object with optional data.
    pub fn create_index_buffer(&self,
                               setup: IndexBufferSetup,
                               data: Option<&[u8]>)
                               -> Result<IndexBufferHandle> {
        if let Some(buf) = data.as_ref() {
            if buf.len() > setup.len() {
                bail!("out of bounds");
            }
        }

        let handle = self.index_buffers.write().unwrap().create().into();

        {
            let mut frame = self.frames.front();
            let ptr = data.map(|v| frame.buf.extend_from_slice(v));
            let task = PreFrameTask::CreateIndexBuffer(handle, setup, ptr);
            frame.pre.push(task);
        }

        Ok(handle)
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_index_buffer(&self,
                               handle: IndexBufferHandle,
                               offset: usize,
                               data: &[u8])
                               -> Result<()> {
        if self.index_buffers.read().unwrap().is_alive(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            let task = PreFrameTask::UpdateIndexBuffer(handle, offset, ptr);
            frame.pre.push(task);
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Deletes a vertex buffer object.
    pub fn delete_index_buffer(&self, handle: IndexBufferHandle) {
        if self.index_buffers.write().unwrap().free(handle) {
            let task = PostFrameTask::DeleteIndexBuffer(handle);
            self.frames.front().post.push(task);
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