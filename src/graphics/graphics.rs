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
use super::bucket::BucketTask;
use super::window::Window;
use super::assets::texture_loader::{TextureLoader, TextureParser, TextureState};

#[derive(Debug, Copy, Clone, Default)]
pub struct GraphicsFrameInfo {
    pub duration: Duration,
    pub drawcall: usize,
    pub vertices: usize,
    pub alive_surfaces: usize,
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
    last_framebuffer_dimensions: (u32, u32),
}

impl GraphicsSystem {
    /// Create a new `GraphicsSystem` with one `Window` context.
    pub fn new(window: Arc<window::Window>, resource: Arc<ResourceSystemShared>) -> Result<Self> {
        let device = unsafe { Device::new() };
        let frames = Arc::new(DoubleFrame::with_capacity(64 * 1024));

        let err = ErrorKind::WindowNotExist;
        let dimensions = window.dimensions().ok_or(err)?;

        let err = ErrorKind::WindowNotExist;
        let point_dimensions = window.point_dimensions().ok_or(err)?;

        let shared =
            GraphicsSystemShared::new(resource, frames.clone(), dimensions, point_dimensions);

        Ok(GraphicsSystem {
               window: window,
               device: device,
               frames: frames,
               shared: Arc::new(shared),
               last_framebuffer_dimensions: dimensions,
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

            let err = ErrorKind::WindowNotExist;
            let dimensions = self.window.dimensions().ok_or(err)?;

            let err = ErrorKind::WindowNotExist;
            let point_dimensions = self.window.point_dimensions().ok_or(err)?;

            if dimensions != self.last_framebuffer_dimensions {
                self.last_framebuffer_dimensions = dimensions;
                self.window.resize(dimensions);
            }

            *self.shared.dimensions.write().unwrap() = (dimensions, point_dimensions);

            {
                self.device.run_one_frame()?;

                {
                    let mut frame = self.frames.back();

                    info.drawcall = 0;
                    info.vertices = 0;
                    for v in &frame.tasks {
                        if let FrameTask::DrawCall(ref dc) = v.1 {
                            info.drawcall += 1;
                            info.vertices += dc.len as usize;
                        }
                    }

                    frame.dispatch(&mut self.device, dimensions)?;
                    frame.clear();
                }
            }

            self.window.swap_buffers()?;

            info.duration = time::Instant::now() - ts;

            {
                let s = &self.shared;
                info.alive_surfaces = Self::clear(&mut s.surfaces.write().unwrap());
                info.alive_shaders = Self::clear(&mut s.shaders.write().unwrap());
                info.alive_frame_buffers = Self::clear(&mut s.framebuffers.write().unwrap());
                info.alive_vertex_buffers = Self::clear(&mut s.vertex_buffers.write().unwrap());

                info.alive_index_buffers = Self::clear(&mut s.index_buffers.write().unwrap());
                info.alive_textures = Self::clear(&mut s.textures.write().unwrap());
                info.alive_render_buffers = Self::clear(&mut s.render_buffers.write().unwrap());
            }

            Ok(info)
        }
    }

    fn clear<T>(v: &mut Registery<T>) -> usize
        where T: Sized
    {
        v.clear();
        v.len()
    }
}

type PipelineState = HashMap<HashValue<str>, usize>;

/// The multi-thread friendly parts of `GraphicsSystem`.
pub struct GraphicsSystemShared {
    resource: Arc<ResourceSystemShared>,
    frames: Arc<DoubleFrame>,
    dimensions: RwLock<((u32, u32), (u32, u32))>,

    surfaces: RwLock<Registery<()>>,
    shaders: RwLock<Registery<PipelineState>>,
    framebuffers: RwLock<Registery<()>>,
    render_buffers: RwLock<Registery<()>>,
    vertex_buffers: RwLock<Registery<()>>,
    index_buffers: RwLock<Registery<()>>,
    textures: RwLock<Registery<Arc<RwLock<TextureState>>>>,
}

impl GraphicsSystemShared {
    /// Create a new `GraphicsSystem` with one `Window` context.
    fn new(resource: Arc<ResourceSystemShared>,
           frames: Arc<DoubleFrame>,
           dimensions: (u32, u32),
           point_dimensions: (u32, u32))
           -> Self {
        GraphicsSystemShared {
            resource: resource,
            frames: frames,
            dimensions: RwLock::new((dimensions, point_dimensions)),

            surfaces: RwLock::new(Registery::new()),
            shaders: RwLock::new(Registery::new()),
            framebuffers: RwLock::new(Registery::new()),
            render_buffers: RwLock::new(Registery::new()),
            vertex_buffers: RwLock::new(Registery::new()),
            index_buffers: RwLock::new(Registery::new()),
            textures: RwLock::new(Registery::new()),
        }
    }

    /// Returns the size in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    /// These are the dimensions of the frame buffer.
    #[inline]
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions.read().unwrap().0
    }

    /// Returns the size in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    #[inline]
    pub fn point_dimensions(&self) -> (u32, u32) {
        self.dimensions.read().unwrap().1
    }

    /// Submit a task into named bucket.
    ///
    /// Tasks inside bucket will be executed in sequential order.
    pub fn submit<'a, T>(&self, surface: SurfaceHandle, task: T) -> Result<()>
        where T: Into<BucketTask<'a>>
    {
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            bail!("Undefined surface handle.");
        }

        match task.into() {
            BucketTask::DrawCall(dc) => self.submit_drawcall(surface, dc),
            BucketTask::VertexBufferUpdate(vbu) => self.submit_update_vertex_buffer(surface, vbu),
            BucketTask::IndexBufferUpdate(ibu) => self.submit_update_index_buffer(surface, ibu),
            BucketTask::TextureUpdate(tu) => self.submit_update_texture(surface, tu),
            BucketTask::SetScissor(sc) => self.submit_set_scissor(surface, sc),
        }
    }

    fn submit_drawcall<'a>(&self,
                           surface: SurfaceHandle,
                           dc: bucket::BucketDrawCall<'a>)
                           -> Result<()> {
        if !self.vertex_buffers.read().unwrap().is_alive(dc.vbo.into()) {
            bail!("Undefined vertex buffer handle.");
        }

        if let Some(ib) = dc.ibo {
            if !self.index_buffers.read().unwrap().is_alive(ib.into()) {
                bail!("Undefined index buffer handle.");
            }
        }

        let mut frame = self.frames.front();

        let uniforms = {
            let mut pack = [None; MAX_UNIFORM_VARIABLES];
            let mut len = 0;

            if let Some(shader) = self.shaders.read().unwrap().get(dc.shader.into()) {
                for &(n, v) in dc.uniforms {
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
            shader: dc.shader,
            uniforms: uniforms,
            vb: dc.vbo,
            ib: dc.ibo,
            primitive: dc.primitive,
            from: dc.from,
            len: dc.len,
        };

        frame.tasks.push((surface, FrameTask::DrawCall(dc)));
        Ok(())
    }

    fn submit_set_scissor(&self,
                          surface: SurfaceHandle,
                          su: bucket::BucketScissorUpdate)
                          -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            bail!("Undefined surface handle.");
        }

        let mut frame = self.frames.front();
        let task = FrameTask::UpdateSurface(su.scissor);
        frame.tasks.push((surface, task));
        Ok(())
    }

    fn submit_update_vertex_buffer(&self,
                                   surface: SurfaceHandle,
                                   vbu: bucket::BucketVertexBufferUpdate)
                                   -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            bail!("Undefined surface handle.");
        }

        if self.vertex_buffers.read().unwrap().is_alive(vbu.vbo.into()) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(vbu.data);
            let task = FrameTask::UpdateVertexBuffer(vbu.vbo, vbu.offset, ptr);
            frame.tasks.push((surface, task));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    fn submit_update_index_buffer(&self,
                                  surface: SurfaceHandle,
                                  ibu: bucket::BucketIndexBufferUpdate)
                                  -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            bail!("Undefined surface handle.");
        }

        if self.index_buffers.read().unwrap().is_alive(ibu.ibo.into()) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(ibu.data);
            let task = FrameTask::UpdateIndexBuffer(ibu.ibo, ibu.offset, ptr);
            frame.tasks.push((surface, task));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    fn submit_update_texture(&self,
                             surface: SurfaceHandle,
                             tu: bucket::BucketTextureUpdate)
                             -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            bail!("Undefined surface handle.");
        }

        if let Some(state) = self.textures.read().unwrap().get(tu.texture.into()) {
            if TextureState::Ready == *state.read().unwrap() {
                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(tu.data);
                let task = FrameTask::UpdateTexture(tu.texture, tu.rect, ptr);
                frame.tasks.push((surface, task));
            }

            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }
}

impl GraphicsSystemShared {
    /// Creates an view with `SurfaceSetup`.
    pub fn create_surface(&self, setup: SurfaceSetup) -> Result<SurfaceHandle> {
        let location = resource::Location::unique("");
        let handle = self.surfaces.write().unwrap().create(location, ()).into();

        {
            let task = PreFrameTask::CreateSurface(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Delete view state object.
    pub fn delete_surface(&self, handle: SurfaceHandle) {
        if self.surfaces
               .write()
               .unwrap()
               .dec_rc(handle.into(), true)
               .is_some() {
            let task = PostFrameTask::DeleteSurface(handle);
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
               .dec_rc(handle.into(), true)
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
               .dec_rc(handle.into(), true)
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
               .dec_rc(handle.into(), true)
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
                                vbo: VertexBufferHandle,
                                offset: usize,
                                data: &[u8])
                                -> Result<()> {
        if self.vertex_buffers.read().unwrap().is_alive(vbo.into()) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            let task = PreFrameTask::UpdateVertexBuffer(vbo, offset, ptr);
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
               .dec_rc(handle.into(), true)
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
                               ibo: IndexBufferHandle,
                               offset: usize,
                               data: &[u8])
                               -> Result<()> {
        if self.index_buffers.read().unwrap().is_alive(ibo.into()) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            let task = PreFrameTask::UpdateIndexBuffer(ibo, offset, ptr);
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
               .dec_rc(handle.into(), true)
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
    pub fn update_texture(&self, texture: TextureHandle, rect: Rect, data: &[u8]) -> Result<()> {
        if let Some(state) = self.textures.read().unwrap().get(texture.into()) {
            if TextureState::Ready == *state.read().unwrap() {
                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(data);
                let task = PreFrameTask::UpdateTexture(texture, rect, ptr);
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
               .dec_rc(handle.into(), true)
               .is_some() {
            let task = PostFrameTask::DeleteTexture(handle);
            self.frames.front().post.push(task);
        }
    }
}