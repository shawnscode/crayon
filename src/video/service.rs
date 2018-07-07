//! The centralized management of video sub-system.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use math;
use resource::prelude::*;
use resource::utils::prelude::*;

use video::assets::prelude::*;
use video::errors::{Error, Result};

use super::assets::mesh_loader::{MeshLoader, MeshParser};
use super::assets::texture_loader::{TextureLoader, TextureParser};

use super::backend::device::Device;
use super::backend::frame::*;
use super::command::*;
use super::window::Window;

/// The information of video module during last frame.
#[derive(Debug, Copy, Clone, Default)]
pub struct VideoFrameInfo {
    pub duration: Duration,
    pub drawcall: u32,
    pub triangles: u32,
    pub alive_surfaces: u32,
    pub alive_shaders: u32,
    pub alive_meshes: u32,
    pub alive_textures: u32,
}

/// The centralized management of video sub-system.
pub struct VideoSystem {
    window: Arc<Window>,
    device: Device,
    frames: Arc<DoubleFrame>,
    shared: Arc<VideoSystemShared>,

    last_dimensions: (u32, u32),
    last_hidpi: f32,
}

impl VideoSystem {
    /// Create a new `VideoSystem` with one `Window` context.
    pub fn new(window: Arc<Window>, resource: Arc<ResourceSystemShared>) -> Result<Self> {
        let device = unsafe { Device::new() };
        let frames = Arc::new(DoubleFrame::with_capacity(64 * 1024));

        let dimensions = window.dimensions().ok_or(Error::WindowNotExists)?;
        let dimensions_in_pixels = window.dimensions_in_pixels().ok_or(Error::WindowNotExists)?;

        let shared =
            VideoSystemShared::new(resource, frames.clone(), dimensions, dimensions_in_pixels);

        Ok(VideoSystem {
            last_dimensions: dimensions,
            last_hidpi: window.hidpi_factor(),

            window: window,
            device: device,
            frames: frames,
            shared: Arc::new(shared),
        })
    }

    /// Returns the multi-thread friendly parts of `VideoSystem`.
    pub fn shared(&self) -> Arc<VideoSystemShared> {
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
    pub fn advance(&mut self) -> Result<VideoFrameInfo> {
        use std::time;

        unsafe {
            let ts = time::Instant::now();

            let dimensions = self.window.dimensions().ok_or(Error::WindowNotExists)?;
            let dimensions_in_pixels = self.window
                .dimensions_in_pixels()
                .ok_or(Error::WindowNotExists)?;

            let hidpi = self.window.hidpi_factor();

            // Resize the window, which would recreate the underlying framebuffer.
            if dimensions != self.last_dimensions
                || (self.last_hidpi - hidpi).abs() > ::std::f32::EPSILON
            {
                self.last_dimensions = dimensions;
                self.last_hidpi = hidpi;
                self.window.resize(dimensions);
            }

            *self.shared.dimensions.write().unwrap() = (dimensions, dimensions_in_pixels);

            {
                self.device.run_one_frame()?;

                {
                    let mut frame = self.frames.back();
                    frame.dispatch(&mut self.device, dimensions, hidpi)?;
                    frame.clear();
                }
            }

            self.window.swap_buffers()?;
            let mut info = VideoFrameInfo::default();
            {
                let v = self.device.frame_info();
                info.drawcall = v.drawcall;
                info.triangles = v.triangles;
            }

            {
                let s = &self.shared;
                info.alive_surfaces = Self::clear(&mut s.surfaces.write().unwrap());
                info.alive_shaders = Self::clear(&mut s.shaders.write().unwrap());
                info.alive_meshes = Self::clear(&mut s.meshes.write().unwrap());
                info.alive_textures = Self::clear(&mut s.textures.write().unwrap());
            }

            info.duration = time::Instant::now() - ts;
            Ok(info)
        }
    }

    fn clear<T>(v: &mut Registery<T>) -> u32
    where
        T: Sized,
    {
        v.clear();
        v.len() as u32
    }
}

/// The multi-thread friendly parts of `VideoSystem`.
pub struct VideoSystemShared {
    resource: Arc<ResourceSystemShared>,
    frames: Arc<DoubleFrame>,
    dimensions: RwLock<((u32, u32), (u32, u32))>,

    surfaces: RwLock<Registery<SurfaceParams>>,
    render_textures: RwLock<Registery<RenderTextureParams>>,
    shaders: RwLock<Registery<ShaderParams>>,
    meshes: Arc<RwLock<Registery<AssetMeshState>>>,
    textures: Arc<RwLock<Registery<AssetTextureState>>>,
}

impl VideoSystemShared {
    /// Create a new `VideoSystem` with one `Window` context.
    fn new(
        resource: Arc<ResourceSystemShared>,
        frames: Arc<DoubleFrame>,
        dimensions: (u32, u32),
        dimensions_in_pixels: (u32, u32),
    ) -> Self {
        VideoSystemShared {
            resource: resource,
            frames: frames,
            dimensions: RwLock::new((dimensions, dimensions_in_pixels)),

            surfaces: RwLock::new(Registery::passive()),
            shaders: RwLock::new(Registery::passive()),
            meshes: Arc::new(RwLock::new(Registery::passive())),
            textures: Arc::new(RwLock::new(Registery::passive())),
            render_textures: RwLock::new(Registery::passive()),
        }
    }

    /// Returns the size in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    #[inline]
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions.read().unwrap().0
    }

    /// Returns the size in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    /// These are the dimensions of the frame buffer.
    #[inline]
    pub fn dimensions_in_pixels(&self) -> (u32, u32) {
        self.dimensions.read().unwrap().1
    }

    /// Submit a task into named bucket.
    ///
    /// Tasks inside bucket will be executed in sequential order.
    pub fn submit<'a, T1, T2>(&self, s: SurfaceHandle, o: T1, task: T2) -> Result<()>
    where
        T1: Into<u64>,
        T2: Into<Command<'a>>,
    {
        if !self.surfaces.read().unwrap().is_alive(s) {
            return Err(Error::SurfaceHandleInvalid(s));
        }

        let o = o.into();
        match task.into() {
            Command::DrawCall(dc) => self.submit_drawcall(s, o, &dc),
            Command::VertexBufferUpdate(vbu) => self.submit_update_vertex_buffer(s, o, &vbu),
            Command::IndexBufferUpdate(ibu) => self.submit_update_index_buffer(s, o, &ibu),
            Command::TextureUpdate(tu) => self.submit_update_texture(s, o, &tu),
            Command::SetScissor(sc) => self.submit_set_scissor(s, o, &sc),
            Command::SetViewport(vp) => self.submit_set_viewport(s, o, &vp),
        }
    }

    fn submit_drawcall<'a>(
        &self,
        surface: SurfaceHandle,
        order: u64,
        dc: &SliceDrawCall<'a>,
    ) -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.meshes.read().unwrap().get(dc.mesh) {
            if !state.is_ready() {
                return Ok(());
            }
        } else {
            return Err(Error::MeshHandleInvalid(dc.mesh));
        }

        let mut frame = self.frames.front();
        let uniforms = {
            let mut pack = Vec::new();
            if let Some(params) = self.shaders.read().unwrap().get(dc.shader) {
                for &(n, v) in dc.uniforms {
                    if let Some(tt) = params.uniforms.variable_type(n) {
                        if tt == v.variable_type() {
                            pack.push((n, frame.buf.extend(&v)));
                        } else {
                            let name = params.uniforms.variable_name(n).unwrap();
                            return Err(Error::DrawFailure(format!(
                                "Unmatched uniform variable: [{:?}]{:?} ({:?} required).",
                                v.variable_type(),
                                name,
                                tt
                            )));
                        }
                    } else {
                        return Err(Error::DrawFailure(format!(
                            "Undefined uniform variable: {:?}.",
                            n
                        )));
                    }
                }
            } else {
                return Err(Error::ShaderHandleInvalid(dc.shader));
            }

            frame.buf.extend_from_slice(&pack)
        };

        let dc = FrameDrawCall {
            shader: dc.shader,
            uniforms: uniforms,
            mesh: dc.mesh,
            index: dc.index,
        };

        frame.tasks.push((surface, order, FrameTask::DrawCall(dc)));
        Ok(())
    }

    fn submit_set_scissor(
        &self,
        surface: SurfaceHandle,
        order: u64,
        su: &ScissorUpdate,
    ) -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        let mut frame = self.frames.front();
        let task = FrameTask::UpdateSurfaceScissor(*su);
        frame.tasks.push((surface, order, task));
        Ok(())
    }

    fn submit_set_viewport(
        &self,
        surface: SurfaceHandle,
        order: u64,
        vp: &ViewportUpdate,
    ) -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        let mut frame = self.frames.front();
        let task = FrameTask::UpdateSurfaceViewport(*vp);
        frame.tasks.push((surface, order, task));
        Ok(())
    }

    fn submit_update_vertex_buffer(
        &self,
        surface: SurfaceHandle,
        order: u64,
        vbu: &VertexBufferUpdate,
    ) -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.meshes.read().unwrap().get(vbu.mesh) {
            if !state.is_ready() {
                unreachable!();
            } else {
                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(vbu.data);
                let task = FrameTask::UpdateVertexBuffer(vbu.mesh, vbu.offset, ptr);
                frame.tasks.push((surface, order, task));
                Ok(())
            }
        } else {
            Err(Error::MeshHandleInvalid(vbu.mesh))
        }
    }

    fn submit_update_index_buffer(
        &self,
        surface: SurfaceHandle,
        order: u64,
        ibu: &IndexBufferUpdate,
    ) -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.meshes.read().unwrap().get(ibu.mesh) {
            if !state.is_ready() {
                unreachable!();
            } else {
                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(ibu.data);
                let task = FrameTask::UpdateIndexBuffer(ibu.mesh, ibu.offset, ptr);
                frame.tasks.push((surface, order, task));
                Ok(())
            }
        } else {
            Err(Error::MeshHandleInvalid(ibu.mesh))
        }
    }

    fn submit_update_texture(
        &self,
        surface: SurfaceHandle,
        order: u64,
        tu: &TextureUpdate,
    ) -> Result<()> {
        if !self.surfaces.read().unwrap().is_alive(surface) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.textures.read().unwrap().get(tu.texture) {
            if !state.is_ready() {
                unreachable!();
            } else {
                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(tu.data);
                let task = FrameTask::UpdateTexture(tu.texture, tu.rect, ptr);
                frame.tasks.push((surface, order, task));
                Ok(())
            }
        } else {
            Err(Error::TextureHandleInvalid(tu.texture))
        }
    }
}

impl VideoSystemShared {
    /// Creates an view with `SurfaceSetup`.
    pub fn create_surface(&self, setup: SurfaceSetup) -> Result<SurfaceHandle> {
        let location = Location::unique("");
        let handle = self.surfaces
            .write()
            .unwrap()
            .create(location, setup)
            .into();

        {
            let task = PreFrameTask::CreateSurface(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Gets the `SurfaceParams` if available.
    pub fn surface(&self, handle: MeshHandle) -> Option<SurfaceParams> {
        self.surfaces.read().unwrap().get(handle).cloned()
    }

    /// Returns true if shader is exists.
    pub fn is_surface_alive(&self, handle: SurfaceHandle) -> bool {
        self.surfaces.read().unwrap().is_alive(handle)
    }

    /// Delete surface object.
    pub fn delete_surface(&self, handle: SurfaceHandle) {
        if self.surfaces.write().unwrap().dec_rc(handle).is_some() {
            let task = PostFrameTask::DeleteSurface(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl VideoSystemShared {
    /// Lookup shader object from location.
    pub fn lookup_shader(&self, location: Location) -> Option<ShaderHandle> {
        self.shaders
            .read()
            .unwrap()
            .lookup(location)
            .map(|v| v.into())
    }

    /// Create a shader with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_shader(&self, setup: ShaderSetup) -> Result<ShaderHandle> {
        setup.validate()?;

        let handle = {
            let mut shaders = self.shaders.write().unwrap();
            let location = setup.location;
            if let Some(handle) = shaders.lookup(location) {
                shaders.inc_rc(handle);
                return Ok(handle.into());
            }

            shaders.create(location, setup.params.clone()).into()
        };

        let task = PreFrameTask::CreatePipeline(handle, setup.params, setup.vs, setup.fs);
        self.frames.front().pre.push(task);
        Ok(handle)
    }

    /// Gets the `ShaderParams` if available.
    pub fn shader(&self, handle: MeshHandle) -> Option<ShaderParams> {
        self.shaders.read().unwrap().get(handle).cloned()
    }

    /// Returns true if shader is exists.
    pub fn is_shader_alive(&self, handle: ShaderHandle) -> bool {
        self.shaders.read().unwrap().is_alive(handle)
    }

    /// Delete shader state object.
    pub fn delete_shader(&self, handle: ShaderHandle) {
        if self.shaders.write().unwrap().dec_rc(handle).is_some() {
            let task = PostFrameTask::DeletePipeline(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl VideoSystemShared {
    /// Lookup mesh object from location.
    pub fn lookup_mesh(&self, location: Location) -> Option<MeshHandle> {
        self.meshes
            .read()
            .unwrap()
            .lookup(location)
            .map(|v| v.into())
    }

    /// Create a new mesh object from location.
    pub fn create_mesh_from_file<T>(&self, setup: MeshSetup) -> Result<MeshHandle>
    where
        T: MeshParser + Send + Sync + 'static,
    {
        setup.validate()?;

        let handle = {
            let mut meshes = self.meshes.write().unwrap();
            if let Some(handle) = meshes.lookup(setup.location) {
                meshes.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = AssetState::NotReady;
            meshes.create(setup.location, state).into()
        };

        let loader = MeshLoader::<T>::new(
            handle,
            setup.params,
            self.meshes.clone(),
            self.frames.clone(),
        );

        self.resource.load_async(loader, setup.location.uri());
        Ok(handle)
    }

    /// Create a new mesh object.
    pub fn create_mesh(&self, setup: MeshSetup) -> Result<MeshHandle> {
        setup.validate()?;

        let handle = {
            let mut meshes = self.meshes.write().unwrap();
            if let Some(handle) = meshes.lookup(setup.location) {
                meshes.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = AssetState::ready(setup.params.clone());
            meshes.create(setup.location, state).into()
        };

        let mut frame = self.frames.front();
        let verts_ptr = setup.verts.map(|v| frame.buf.extend_from_slice(v));
        let idxes_ptr = setup.idxes.map(|v| frame.buf.extend_from_slice(v));
        let task = PreFrameTask::CreateMesh(handle, setup.params, verts_ptr, idxes_ptr);
        frame.pre.push(task);
        Ok(handle)
    }

    /// Gets the `MeshParams` if available.
    ///
    /// Notes that this function might returns `None` even if `is_mesh_alive` returns
    /// true. The underlying object might be still in creation process and can not be
    /// provided yet.
    pub fn mesh(&self, handle: MeshHandle) -> Option<Arc<MeshParams>> {
        self.meshes.read().unwrap().get(handle).and_then(|v| {
            if let &AssetState::Ready(ref mso) = v {
                Some(mso.clone())
            } else {
                None
            }
        })
    }

    /// Checks whether the mesh is exists.
    pub fn is_mesh_alive(&self, handle: MeshHandle) -> bool {
        self.meshes.read().unwrap().is_alive(handle)
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(&self, mesh: MeshHandle, offset: usize, data: &[u8]) -> Result<()> {
        if let Some(state) = self.meshes.read().unwrap().get(mesh) {
            if let &AssetState::Ready(ref mso) = state {
                if mso.hint == MeshHint::Immutable {
                    return Err(Error::UpdateImmutableBuffer);
                }

                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(data);
                let task = PreFrameTask::UpdateVertexBuffer(mesh, offset, ptr);
                frame.pre.push(task);
            } else {
                unreachable!();
            }

            Ok(())
        } else {
            Err(Error::MeshHandleInvalid(mesh))
        }
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_index_buffer(&self, mesh: MeshHandle, offset: usize, data: &[u8]) -> Result<()> {
        if let Some(state) = self.meshes.read().unwrap().get(mesh) {
            if let &AssetState::Ready(ref mso) = state {
                if mso.hint == MeshHint::Immutable {
                    return Err(Error::UpdateImmutableBuffer);
                }

                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(data);
                let task = PreFrameTask::UpdateIndexBuffer(mesh, offset, ptr);
                frame.pre.push(task);
            } else {
                unreachable!();
            }

            Ok(())
        } else {
            Err(Error::MeshHandleInvalid(mesh))
        }
    }

    /// Delete mesh object.
    pub fn delete_mesh(&self, mesh: MeshHandle) {
        if self.meshes.write().unwrap().dec_rc(mesh).is_some() {
            let task = PostFrameTask::DeleteMesh(mesh);
            self.frames.front().post.push(task);
        }
    }
}

impl VideoSystemShared {
    /// Lookup texture object from location.
    pub fn lookup_texture(&self, location: Location) -> Option<TextureHandle> {
        self.textures
            .read()
            .unwrap()
            .lookup(location)
            .map(|v| v.into())
    }

    /// Create texture object from location.
    pub fn create_texture_from_file<T>(&self, setup: TextureSetup) -> Result<TextureHandle>
    where
        T: TextureParser + Send + Sync + 'static,
    {
        setup.validate()?;

        let handle = {
            let mut textures = self.textures.write().unwrap();
            if let Some(handle) = textures.lookup(setup.location) {
                textures.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = AssetState::NotReady;
            let handle = textures.create(setup.location, state).into();
            handle
        };

        let loader = TextureLoader::<T>::new(
            handle,
            setup.params,
            self.textures.clone(),
            self.frames.clone(),
        );

        self.resource.load_async(loader, setup.location.uri());
        Ok(handle)
    }

    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture(&self, setup: TextureSetup) -> Result<TextureHandle> {
        setup.validate()?;

        let handle = {
            let mut textures = self.textures.write().unwrap();
            if let Some(handle) = textures.lookup(setup.location) {
                textures.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = AssetState::ready(setup.params.clone());
            textures.create(setup.location, state).into()
        };

        let mut frame = self.frames.front();
        let ptr = setup.data.map(|v| frame.buf.extend_from_slice(v));
        let task = PreFrameTask::CreateTexture(handle, setup.params, ptr);
        frame.pre.push(task);
        Ok(handle)
    }

    /// Gets the `TextureParams` if available.
    ///
    /// Notes that this function might returns `None` even if `is_texture_alive` returns
    /// true. The underlying object might be still in creation process and can not be
    /// provided yet.
    pub fn texture(&self, handle: TextureHandle) -> Option<TextureParams> {
        self.textures.read().unwrap().get(handle).and_then(|v| {
            if let &AssetState::Ready(ref texture) = v {
                Some(*texture.as_ref())
            } else {
                None
            }
        })
    }

    /// Returns true if texture is exists.
    pub fn is_texture_alive(&self, handle: TextureHandle) -> bool {
        self.textures.read().unwrap().is_alive(handle)
    }

    /// Update a contiguous subregion of an existing two-dimensional texture object.
    pub fn update_texture(
        &self,
        handle: TextureHandle,
        rect: math::Aabb2<f32>,
        data: &[u8],
    ) -> Result<()> {
        if let Some(state) = self.textures.read().unwrap().get(handle) {
            if let AssetState::Ready(ref texture) = *state {
                if texture.hint == TextureHint::Immutable {
                    return Err(Error::UpdateImmutableBuffer);
                }

                let mut frame = self.frames.front();
                let ptr = frame.buf.extend_from_slice(data);
                let task = PreFrameTask::UpdateTexture(handle, rect, ptr);
                frame.pre.push(task);
            } else {
                unreachable!()
            }

            Ok(())
        } else {
            Err(Error::TextureHandleInvalid(handle))
        }
    }

    /// Delete the texture object.
    pub fn delete_texture(&self, handle: TextureHandle) {
        if self.textures.write().unwrap().dec_rc(handle).is_some() {
            let task = PostFrameTask::DeleteTexture(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl VideoSystemShared {
    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(&self, setup: RenderTextureSetup) -> Result<RenderTextureHandle> {
        let location = Location::unique("");
        let handle = self.render_textures
            .write()
            .unwrap()
            .create(location, setup)
            .into();

        {
            let task = PreFrameTask::CreateRenderTexture(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Gets the `RenderTextureParams` if available.
    pub fn render_texture(&self, handle: MeshHandle) -> Option<RenderTextureParams> {
        self.render_textures.read().unwrap().get(handle).cloned()
    }

    /// Returns true if texture is exists.
    pub fn is_render_texture_alive(&self, handle: RenderTextureHandle) -> bool {
        self.render_textures.read().unwrap().is_alive(handle)
    }

    /// Delete the render texture object.
    pub fn delete_render_texture(&self, handle: RenderTextureHandle) {
        if self.render_textures
            .write()
            .unwrap()
            .dec_rc(handle)
            .is_some()
        {
            let task = PostFrameTask::DeleteRenderTexture(handle);
            self.frames.front().post.push(task);
        }
    }
}
