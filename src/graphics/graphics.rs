//! The centralized management of video sub-system.

use std::sync::{Arc, RwLock, Mutex, MutexGuard};
use std::time::Duration;
use std::marker::PhantomData;
use std::path::Path;

use utils::{Rect, ObjectPool};
use resource;
use resource::ResourceSystemShared;

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
    pub fn new(window: Arc<window::Window>, resource: Arc<ResourceSystemShared>) -> Result<Self> {
        let device = unsafe { Device::new() };
        let frames = Arc::new(DoubleFrame::with_capacity(64 * 1024));

        let shared = GraphicsSystemShared::new(resource, frames.clone());

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

            info.alive_views = self.shared.views.read().unwrap().len();
            info.alive_pipelines = self.shared.pipelines.read().unwrap().len();
            info.alive_frame_buffers = self.shared.framebuffers.read().unwrap().len();
            info.alive_vertex_buffers = self.shared.vertex_buffers.read().unwrap().len();
            info.alive_index_buffers = self.shared.index_buffers.read().unwrap().len();
            info.alive_textures = self.shared.textures.read().unwrap().len();
            info.alive_render_buffers = self.shared.render_buffers.read().unwrap().len();

            Ok(info)
        }
    }
}

/// Unique resource label.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResourceLabel(u64);

impl ResourceLabel {
    #[inline(always)]
    pub fn new(v: u64) -> Self {
        ResourceLabel(v)
    }
}

/// The multi-thread friendly parts of `GraphicsSystem`.
pub struct GraphicsSystemShared {
    resource: Arc<ResourceSystemShared>,
    frames: Arc<DoubleFrame>,
    label: RwLock<u64>,

    views: RwLock<ObjectPool<ResourceLabel>>,
    pipelines: RwLock<ObjectPool<ResourceLabel>>,
    framebuffers: RwLock<ObjectPool<ResourceLabel>>,
    vertex_buffers: RwLock<ObjectPool<ResourceLabel>>,
    index_buffers: RwLock<ObjectPool<ResourceLabel>>,
    render_buffers: RwLock<ObjectPool<ResourceLabel>>,

    textures: RwLock<ObjectPool<(ResourceLabel, Arc<RwLock<TextureState>>)>>,
}

impl GraphicsSystemShared {
    /// Create a new `GraphicsSystem` with one `Window` context.
    fn new(resource: Arc<ResourceSystemShared>, frames: Arc<DoubleFrame>) -> Self {
        GraphicsSystemShared {
            resource: resource,
            frames: frames,
            label: RwLock::new(0),

            views: RwLock::new(ObjectPool::new()),
            pipelines: RwLock::new(ObjectPool::new()),
            framebuffers: RwLock::new(ObjectPool::new()),
            vertex_buffers: RwLock::new(ObjectPool::new()),
            index_buffers: RwLock::new(ObjectPool::new()),
            render_buffers: RwLock::new(ObjectPool::new()),
            textures: RwLock::new(ObjectPool::new()),
        }
    }

    /// Create a fresh and unused `ResourceLabel` in this registery.
    #[inline]
    pub fn create_label(&self) -> ResourceLabel {
        let mut label = self.label.write().unwrap();
        *label = *label + 1;
        ResourceLabel::new(*label)
    }

    /// Remove all resource matching label from registry incrementally.
    pub fn delete_label(&self, label: ResourceLabel) {
        {
            let mut views = self.views.write().unwrap();
            for v in views.free_if(|v| *v == label) {
                let task = PostFrameTask::DeleteView(v.into());
                self.frames.front().post.push(task);
            }
        }

        {
            let mut pipelines = self.pipelines.write().unwrap();
            for v in pipelines.free_if(|v| *v == label) {
                let task = PostFrameTask::DeletePipeline(v.into());
                self.frames.front().post.push(task);
            }
        }

        {
            let mut framebuffers = self.framebuffers.write().unwrap();
            for v in framebuffers.free_if(|v| *v == label) {
                let task = PostFrameTask::DeleteFrameBuffer(v.into());
                self.frames.front().post.push(task);
            }
        }

        {
            let mut render_buffers = self.render_buffers.write().unwrap();
            for v in render_buffers.free_if(|v| *v == label) {
                let task = PostFrameTask::DeleteRenderBuffer(v.into());
                self.frames.front().post.push(task);
            }
        }

        {
            let mut vertex_buffers = self.vertex_buffers.write().unwrap();
            for v in vertex_buffers.free_if(|v| *v == label) {
                let task = PostFrameTask::DeleteVertexBuffer(v.into());
                self.frames.front().post.push(task);
            }
        }

        {
            let mut index_buffers = self.index_buffers.write().unwrap();
            for v in index_buffers.free_if(|v| *v == label) {
                let task = PostFrameTask::DeleteIndexBuffer(v.into());
                self.frames.front().post.push(task);
            }
        }

        {
            let mut textures = self.textures.write().unwrap();
            for v in textures.free_if(|v| v.0 == label) {
                let task = PostFrameTask::DeleteTexture(v.into());
                self.frames.front().post.push(task);
            }
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
    pub fn create_view(&self,
                       label: ResourceLabel,
                       setup: ViewStateSetup)
                       -> Result<ViewStateHandle> {

        let handle = self.views.write().unwrap().create(label).into();

        {
            let task = PreFrameTask::CreateView(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Create a pipeline with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_pipeline(&self,
                           label: ResourceLabel,
                           setup: PipelineStateSetup,
                           vs: String,
                           fs: String)
                           -> Result<PipelineStateHandle> {
        let handle = self.pipelines.write().unwrap().create(label).into();

        {
            let task = PreFrameTask::CreatePipeline(handle, setup, vs, fs);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Create a framebuffer object. A framebuffer allows you to render primitives directly to a texture,
    /// which can then be used in other rendering operations.
    ///
    /// At least one color attachment has been attached before you can use it.
    pub fn create_framebuffer(&self,
                              label: ResourceLabel,
                              setup: FrameBufferSetup)
                              -> Result<FrameBufferHandle> {
        let handle = self.framebuffers.write().unwrap().create(label).into();

        {
            let task = PreFrameTask::CreateFrameBuffer(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Create a render buffer object, which could be attached to framebuffer.
    pub fn create_render_buffer(&self,
                                label: ResourceLabel,
                                setup: RenderBufferSetup)
                                -> Result<RenderBufferHandle> {
        let handle = self.render_buffers.write().unwrap().create(label).into();

        {
            let task = PreFrameTask::CreateRenderBuffer(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Create vertex buffer object with vertex layout declaration and optional data.
    pub fn create_vertex_buffer(&self,
                                label: ResourceLabel,
                                setup: VertexBufferSetup,
                                data: Option<&[u8]>)
                                -> Result<VertexBufferHandle> {
        if let Some(buf) = data.as_ref() {
            if buf.len() > setup.len() {
                bail!("out of bounds");
            }
        }

        let handle = self.vertex_buffers.write().unwrap().create(label).into();

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

    /// Create index buffer object with optional data.
    pub fn create_index_buffer(&self,
                               label: ResourceLabel,
                               setup: IndexBufferSetup,
                               data: Option<&[u8]>)
                               -> Result<IndexBufferHandle> {
        if let Some(buf) = data.as_ref() {
            if buf.len() > setup.len() {
                bail!("out of bounds");
            }
        }

        let handle = self.index_buffers.write().unwrap().create(label).into();

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
}

pub struct Texture {
    pub format: TextureFormat,
    pub dimensions: (u32, u32),
    pub data: Vec<u8>,
}

pub trait TextureParser {
    fn parse(bytes: &[u8]) -> Result<Texture>;
}

impl GraphicsSystemShared {
    /// Create texture object from location.
    pub fn create_texture_from<T, P>(&self,
                                     label: ResourceLabel,
                                     setup: TextureSetup,
                                     location: P)
                                     -> Result<TextureHandle>
        where T: TextureParser + Send + Sync + 'static,
              P: AsRef<Path>
    {
        let state = Arc::new(RwLock::new(TextureState::NotReady));

        let handle = self.textures
            .write()
            .unwrap()
            .create((label, state.clone()))
            .into();

        let loader: TextureLoader<T> = TextureLoader {
            state: state,
            setup: setup,
            handle: handle,
            frames: self.frames.clone(),
            _phantom: PhantomData,
        };

        self.resource.load_async(loader, location);
        Ok(handle)
    }

    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture(&self,
                          label: ResourceLabel,
                          setup: TextureSetup,
                          data: Option<&[u8]>)
                          -> Result<TextureHandle> {
        let state = Arc::new(RwLock::new(TextureState::Ready));
        let handle = self.textures.write().unwrap().create((label, state)).into();

        {
            let mut frame = self.frames.front();
            let ptr = data.map(|v| frame.buf.extend_from_slice(v));
            let task = PreFrameTask::CreateTexture(handle, setup, ptr);
            frame.pre.push(task);
        }

        Ok(handle)
    }

    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(&self,
                                 label: ResourceLabel,
                                 setup: RenderTextureSetup)
                                 -> Result<TextureHandle> {
        let state = Arc::new(RwLock::new(TextureState::Ready));
        let handle = self.textures.write().unwrap().create((label, state)).into();

        {
            let task = PreFrameTask::CreateRenderTexture(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Update the texture.
    ///
    /// Notes that this method might fails without any error when the texture is not
    /// ready for operating.
    pub fn update_texture(&self, handle: TextureHandle, rect: Rect, data: &[u8]) -> Result<()> {
        if let Some(&(_, ref state)) = self.textures.read().unwrap().get(handle) {
            if TextureState::Ready == *state.read().unwrap() {
                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(data);
                let task = PreFrameTask::UpdateTexture(handle, rect, ptr);
                frame.pre.push(task);
            }

            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }
}

#[derive(PartialEq, Eq)]
enum TextureState {
    NotReady,
    Ready,
    Err(String),
}

struct TextureLoader<T>
    where T: TextureParser
{
    handle: TextureHandle,
    setup: TextureSetup,
    state: Arc<RwLock<TextureState>>,
    frames: Arc<DoubleFrame>,
    _phantom: PhantomData<T>,
}

impl<T> resource::ResourceAsyncLoader for TextureLoader<T>
    where T: TextureParser + Send + Sync + 'static
{
    fn on_finished(&mut self, path: &Path, result: resource::errors::Result<&[u8]>) {
        let state = match result {
            Ok(bytes) => {
                match T::parse(bytes) {
                    Ok(texture) => {
                        self.setup.dimensions = texture.dimensions;
                        self.setup.format = texture.format;

                        let mut frame = self.frames.front();
                        let ptr = frame.buf.extend_from_slice(&texture.data);
                        let task = PreFrameTask::CreateTexture(self.handle, self.setup, Some(ptr));
                        frame.pre.push(task);
                        TextureState::Ready
                    }
                    Err(error) => {
                        let error = format!("Failed to load texture at {:?}.\n{:?}", path, error);
                        TextureState::Err(error)
                    }
                }
            }
            Err(error) => {
                let error = format!("Failed to load texture at {:?}.\n{:?}", path, error);
                TextureState::Err(error)
            }
        };

        *self.state.write().unwrap() = state;
    }
}

struct DoubleFrame {
    idx: RwLock<usize>,
    frames: [Mutex<Frame>; 2],
}

impl DoubleFrame {
    fn with_capacity(capacity: usize) -> Self {
        DoubleFrame {
            idx: RwLock::new(0),
            frames: [Mutex::new(Frame::with_capacity(capacity)),
                     Mutex::new(Frame::with_capacity(capacity))],
        }
    }

    #[inline]
    fn front(&self) -> MutexGuard<Frame> {
        self.frames[*self.idx.read().unwrap()].lock().unwrap()
    }

    #[inline]
    fn back(&self) -> MutexGuard<Frame> {
        self.frames[(*self.idx.read().unwrap() + 1) % 2]
            .lock()
            .unwrap()
    }

    #[inline]
    fn swap_frames(&self) {
        let mut idx = self.idx.write().unwrap();
        *idx = (*idx + 1) % 2;
    }
}