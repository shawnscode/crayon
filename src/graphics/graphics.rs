//! The centralized management of video sub-system.

use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::collections::HashMap;

use utils::{Rect, HashValue};
use resource;
use resource::{ResourceSystemShared, Registery};

use super::*;
use super::errors::*;
use super::backend::frame::*;
use super::backend::device::Device;
use super::window::Window;
use super::assets::texture_loader::{TextureLoader, TextureParser, TextureState};

#[derive(Debug, Copy, Clone, Default)]
pub struct GraphicsFrameInfo {
    pub duration: Duration,
    pub drawcall: usize,
    pub alive_views: usize,
    pub alive_shaders: usize,
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
            info.alive_shaders = self.shared.shaders.read().unwrap().len();
            info.alive_frame_buffers = self.shared.framebuffers.read().unwrap().len();
            info.alive_vertex_buffers = self.shared.vertex_buffers.read().unwrap().len();
            info.alive_index_buffers = self.shared.index_buffers.read().unwrap().len();
            info.alive_textures = self.shared.textures.read().unwrap().len();
            info.alive_render_buffers = self.shared.render_buffers.read().unwrap().len();

            Ok(info)
        }
    }
}

type PipelineState = HashMap<HashValue<str>, usize>;

/// The multi-thread friendly parts of `GraphicsSystem`.
pub struct GraphicsSystemShared {
    resource: Arc<ResourceSystemShared>,
    frames: Arc<DoubleFrame>,

    views: RwLock<Registery<()>>,
    shaders: RwLock<Registery<PipelineState>>,
    framebuffers: RwLock<Registery<()>>,
    render_buffers: RwLock<Registery<()>>,
    vertex_buffers: RwLock<Registery<()>>,
    index_buffers: RwLock<Registery<()>>,

    textures: RwLock<Registery<Arc<RwLock<TextureState>>>>,
}

impl GraphicsSystemShared {
    /// Create a new `GraphicsSystem` with one `Window` context.
    fn new(resource: Arc<ResourceSystemShared>, frames: Arc<DoubleFrame>) -> Self {
        GraphicsSystemShared {
            resource: resource,
            frames: frames,

            views: RwLock::new(Registery::new()),
            shaders: RwLock::new(Registery::new()),
            framebuffers: RwLock::new(Registery::new()),
            render_buffers: RwLock::new(Registery::new()),
            vertex_buffers: RwLock::new(Registery::new()),
            index_buffers: RwLock::new(Registery::new()),
            textures: RwLock::new(Registery::new()),
        }
    }

    /// Make a new draw call.
    pub fn submit(&self,
                  order: u64,
                  view: ViewStateHandle,
                  shader: ShaderHandle,
                  uniforms: &[(HashValue<str>, UniformVariable)],
                  vb: VertexBufferHandle,
                  ib: Option<IndexBufferHandle>,
                  primitive: Primitive,
                  from: u32,
                  len: u32)
                  -> Result<()> {
        if !self.views.read().unwrap().is_alive(view.into()) {
            bail!("Undefined view state handle.");
        }

        if !self.vertex_buffers.read().unwrap().is_alive(vb.into()) {
            bail!("Undefined vertex buffer handle.");
        }

        if let Some(ib) = ib {
            if !self.index_buffers.read().unwrap().is_alive(ib.into()) {
                bail!("Undefined index buffer handle.");
            }
        }

        let mut frame = self.frames.front();

        let uniforms = {
            let mut pack = [None; MAX_UNIFORM_VARIABLES];
            let mut len = 0;

            if let Some(shader) = self.shaders.read().unwrap().get(shader.into()) {
                for &(n, v) in uniforms {
                    if let Some(location) = shader.get(&n) {
                        pack[*location] = Some(frame.buf.extend(&v));
                        len = len.max((*location + 1));
                    } else {
                        bail!(format!("Undefined uniform variable: {:?}.", n));
                    }
                }
            } else {
                bail!("Undefined shader state handle.");
            }

            frame.buf.extend_from_slice(&pack[0..len])
        };

        let dc = FrameDrawCall {
            order: order,
            view: view,
            shader: shader,
            uniforms: uniforms,
            vb: vb,
            ib: ib,
            primitive: primitive,
            from: from,
            len: len,
        };

        frame.drawcalls.push(dc);
        Ok(())
    }
}

impl GraphicsSystemShared {
    /// Creates an view with `ViewStateSetup`.
    pub fn create_view(&self, setup: ViewStateSetup) -> Result<ViewStateHandle> {
        let location = resource::Location::unique("");
        let handle = self.views.write().unwrap().create(location, ()).into();

        {
            let task = PreFrameTask::CreateView(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Delete view state object.
    pub fn delete_view(&self, handle: ViewStateHandle) {
        if self.views.write().unwrap().dec_rc(handle.into()).is_some() {
            let task = PostFrameTask::DeleteView(handle);
            self.frames.front().post.push(task);
        }
    }

    /// Create a shader with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_shader(&self, setup: ShaderSetup) -> Result<ShaderHandle> {
        if setup.uniform_variables.len() > MAX_UNIFORM_VARIABLES {
            bail!("Too many uniform variables (>= {:?}).",
                  MAX_UNIFORM_VARIABLES);
        }

        let mut shader = PipelineState::new();
        for (i, v) in setup.uniform_variables.iter().enumerate() {
            let v: HashValue<str> = v.into();
            shader.insert(v, i);
        }

        let loc = resource::Location::unique("");
        let handle = self.shaders.write().unwrap().create(loc, shader).into();

        {
            let task = PreFrameTask::CreatePipeline(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Delete shader state object.
    pub fn delete_shader(&self, handle: ShaderHandle) {
        if self.shaders
               .write()
               .unwrap()
               .dec_rc(handle.into())
               .is_some() {
            let task = PostFrameTask::DeletePipeline(handle);
            self.frames.front().post.push(task);
        }
    }

    /// Create a framebuffer object. A framebuffer allows you to render primitives directly to a texture,
    /// which can then be used in other rendering operations.
    ///
    /// At least one color attachment has been attached before you can use it.
    pub fn create_framebuffer(&self, setup: FrameBufferSetup) -> Result<FrameBufferHandle> {
        let location = resource::Location::unique("");
        let handle = self.framebuffers
            .write()
            .unwrap()
            .create(location, ())
            .into();

        {
            let task = PreFrameTask::CreateFrameBuffer(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Delete frame buffer object.
    pub fn delete_framebuffer(&self, handle: FrameBufferHandle) {
        if self.framebuffers
               .write()
               .unwrap()
               .dec_rc(handle.into())
               .is_some() {
            let task = PostFrameTask::DeleteFrameBuffer(handle);
            self.frames.front().post.push(task);
        }
    }

    /// Create a render buffer object, which could be attached to framebuffer.
    pub fn create_render_buffer(&self, setup: RenderBufferSetup) -> Result<RenderBufferHandle> {
        let location = resource::Location::unique("");
        let handle = self.render_buffers
            .write()
            .unwrap()
            .create(location, ())
            .into();

        {
            let task = PreFrameTask::CreateRenderBuffer(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Delete frame buffer object.
    pub fn delete_render_buffer(&self, handle: RenderBufferHandle) {
        if self.render_buffers
               .write()
               .unwrap()
               .dec_rc(handle.into())
               .is_some() {
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

        let location = resource::Location::unique("");
        let handle = self.vertex_buffers
            .write()
            .unwrap()
            .create(location, ())
            .into();

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
        if self.vertex_buffers.read().unwrap().is_alive(handle.into()) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            let task = PreFrameTask::UpdateVertexBuffer(handle, offset, ptr);
            frame.pre.push(task);
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Delete vertex buffer object.
    pub fn delete_vertex_buffer(&self, handle: VertexBufferHandle) {
        if self.vertex_buffers
               .write()
               .unwrap()
               .dec_rc(handle.into())
               .is_some() {
            let task = PostFrameTask::DeleteVertexBuffer(handle);
            self.frames.front().post.push(task);
        }
    }

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

        let location = resource::Location::unique("");
        let handle = self.index_buffers
            .write()
            .unwrap()
            .create(location, ())
            .into();

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
        if self.index_buffers.read().unwrap().is_alive(handle.into()) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            let task = PreFrameTask::UpdateIndexBuffer(handle, offset, ptr);
            frame.pre.push(task);
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Delete index buffer object.
    pub fn delete_index_buffer(&self, handle: IndexBufferHandle) {
        if self.index_buffers
               .write()
               .unwrap()
               .dec_rc(handle.into())
               .is_some() {
            let task = PostFrameTask::DeleteIndexBuffer(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Lookup texture object from location.
    pub fn lookup_texture_from(&self, location: resource::Location) -> Option<TextureHandle> {
        self.textures
            .read()
            .unwrap()
            .lookup(location)
            .map(|v| v.into())
    }

    /// Create texture object from location.
    pub fn create_texture_from<T>(&self,
                                  location: resource::Location,
                                  setup: TextureSetup)
                                  -> Result<TextureHandle>
        where T: TextureParser + Send + Sync + 'static
    {
        if let Some(v) = self.lookup_texture_from(location) {
            self.textures.write().unwrap().inc_rc(v.into());
            return Ok(v);
        }

        let state = Arc::new(RwLock::new(TextureState::NotReady));
        let handle = {
            let mut textures = self.textures.write().unwrap();
            textures.create(location, state.clone()).into()
        };

        let loader = TextureLoader::<T>::new(handle, state, setup, self.frames.clone());
        self.resource.load_async(loader, location.uri());

        Ok(handle)
    }

    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture(&self,
                          setup: TextureSetup,
                          data: Option<&[u8]>)
                          -> Result<TextureHandle> {
        let loc = resource::Location::unique("");
        let state = Arc::new(RwLock::new(TextureState::Ready));
        let handle = self.textures.write().unwrap().create(loc, state).into();

        {
            let mut frame = self.frames.front();
            let ptr = data.map(|v| frame.buf.extend_from_slice(v));
            let task = PreFrameTask::CreateTexture(handle, setup, ptr);
            frame.pre.push(task);
        }

        Ok(handle)
    }

    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(&self, setup: RenderTextureSetup) -> Result<TextureHandle> {
        let loc = resource::Location::unique("");
        let state = Arc::new(RwLock::new(TextureState::Ready));
        let handle = self.textures.write().unwrap().create(loc, state).into();

        {
            let task = PreFrameTask::CreateRenderTexture(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Update the texture object.
    ///
    /// Notes that this method might fails without any error when the texture is not
    /// ready for operating.
    pub fn update_texture(&self, handle: TextureHandle, rect: Rect, data: &[u8]) -> Result<()> {
        if let Some(state) = self.textures.read().unwrap().get(handle.into()) {
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

    /// Delete the texture object.
    pub fn delete_texture(&self, handle: TextureHandle) {
        if self.textures
               .write()
               .unwrap()
               .dec_rc(handle.into())
               .is_some() {
            let task = PostFrameTask::DeleteTexture(handle);
            self.frames.front().post.push(task);
        }
    }
}