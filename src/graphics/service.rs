//! The centralized management of video sub-system.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::Duration;

use utils::{HashValue, Rect};

use resource::prelude::*;
use resource::utils::prelude::*;

use graphics::MAX_UNIFORM_VARIABLES;
use graphics::errors::{Error, Result};
use graphics::assets::prelude::*;

use super::assets::mesh::MeshStateObject;
use super::assets::shader::ShaderStateObject;
use super::assets::texture::TextureStateObject;
use super::assets::mesh_loader::{MeshLoader, MeshParser};
use super::assets::texture_loader::{TextureLoader, TextureParser};

use super::backend::frame::*;
use super::backend::device::Device;
use super::command::*;
use super::window::Window;

/// The information of graphics module during last frame.
#[derive(Debug, Copy, Clone, Default)]
pub struct GraphicsFrameInfo {
    pub duration: Duration,
    pub drawcall: u32,
    pub triangles: u32,
    pub alive_surfaces: u32,
    pub alive_shaders: u32,
    pub alive_frame_buffers: u32,
    pub alive_meshes: u32,
    pub alive_textures: u32,
    pub alive_render_buffers: u32,
}

/// The centralized management of video sub-system.
pub struct GraphicsSystem {
    window: Arc<Window>,
    device: Device,
    frames: Arc<DoubleFrame>,
    shared: Arc<GraphicsSystemShared>,

    last_dimensions: (u32, u32),
    last_hidpi: f32,
}

impl GraphicsSystem {
    /// Create a new `GraphicsSystem` with one `Window` context.
    pub fn new(window: Arc<Window>, resource: Arc<ResourceSystemShared>) -> Result<Self> {
        let device = unsafe { Device::new() };
        let frames = Arc::new(DoubleFrame::with_capacity(64 * 1024));

        let dimensions = window.dimensions().ok_or(Error::WindowNotExists)?;
        let dimensions_in_pixels = window.dimensions_in_pixels().ok_or(Error::WindowNotExists)?;

        let shared =
            GraphicsSystemShared::new(resource, frames.clone(), dimensions, dimensions_in_pixels);

        Ok(GraphicsSystem {
            last_dimensions: dimensions,
            last_hidpi: window.hidpi_factor(),

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
            let mut info = GraphicsFrameInfo::default();
            {
                let v = self.device.frame_info();
                info.drawcall = v.drawcall;
                info.triangles = v.triangles;
            }

            {
                let s = &self.shared;
                info.alive_surfaces = Self::clear(&mut s.surfaces.write().unwrap());
                info.alive_shaders = Self::clear(&mut s.shaders.write().unwrap());
                info.alive_frame_buffers = Self::clear(&mut s.framebuffers.write().unwrap());
                info.alive_meshes = Self::clear(&mut s.meshes.write().unwrap());
                info.alive_textures = Self::clear(&mut s.textures.write().unwrap());
                info.alive_render_buffers = Self::clear(&mut s.render_buffers.write().unwrap());
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

/// The multi-thread friendly parts of `GraphicsSystem`.
pub struct GraphicsSystemShared {
    resource: Arc<ResourceSystemShared>,
    frames: Arc<DoubleFrame>,
    dimensions: RwLock<((u32, u32), (u32, u32))>,

    surfaces: RwLock<Registery<()>>,
    framebuffers: RwLock<Registery<()>>,
    render_buffers: RwLock<Registery<()>>,
    shaders: RwLock<Registery<AssetShaderState>>,
    meshes: RwLock<Registery<Arc<RwLock<AssetMeshState>>>>,
    textures: RwLock<Registery<Arc<RwLock<AssetTextureState>>>>,
    render_textures: RwLock<Registery<Arc<RwLock<AssetRenderTextureState>>>>,
}

impl GraphicsSystemShared {
    /// Create a new `GraphicsSystem` with one `Window` context.
    fn new(
        resource: Arc<ResourceSystemShared>,
        frames: Arc<DoubleFrame>,
        dimensions: (u32, u32),
        dimensions_in_pixels: (u32, u32),
    ) -> Self {
        GraphicsSystemShared {
            resource: resource,
            frames: frames,
            dimensions: RwLock::new((dimensions, dimensions_in_pixels)),

            surfaces: RwLock::new(Registery::passive()),
            shaders: RwLock::new(Registery::passive()),
            framebuffers: RwLock::new(Registery::passive()),
            render_buffers: RwLock::new(Registery::passive()),
            meshes: RwLock::new(Registery::passive()),
            textures: RwLock::new(Registery::passive()),
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
        if !self.surfaces.read().unwrap().is_alive(s.into()) {
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
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.meshes.read().unwrap().get(dc.mesh.into()) {
            if !state.read().unwrap().is_ready() {
                return Ok(());
            }
        } else {
            return Err(Error::MeshHandleInvalid(dc.mesh));
        }

        let mut frame = self.frames.front();
        let uniforms = {
            let mut pack = Vec::new();
            if let Some(state) = self.shaders.read().unwrap().get(dc.shader.into()) {
                match *state {
                    AssetState::Ready(ref sso) => for &(n, v) in dc.uniforms {
                        if let Some(tt) = sso.uniform_variable(n) {
                            if tt == v.variable_type() {
                                pack.push((n, frame.buf.extend(&v)));
                            } else {
                                let name = sso.uniform_variable_name(n).unwrap();

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
                    },
                    _ => return Ok(()),
                };
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
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
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
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
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
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.meshes.read().unwrap().get(vbu.mesh.into()) {
            if !state.read().unwrap().is_ready() {
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
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.meshes.read().unwrap().get(ibu.mesh.into()) {
            if !state.read().unwrap().is_ready() {
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
        if !self.surfaces.read().unwrap().is_alive(surface.into()) {
            return Err(Error::SurfaceHandleInvalid(surface));
        }

        if let Some(state) = self.textures.read().unwrap().get(tu.texture.into()) {
            if !state.read().unwrap().is_ready() {
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

impl GraphicsSystemShared {
    /// Creates an view with `SurfaceSetup`.
    pub fn create_surface(&self, setup: SurfaceSetup) -> Result<SurfaceHandle> {
        let location = Location::unique("");
        let handle = self.surfaces.write().unwrap().create(location, ()).into();

        {
            let task = PreFrameTask::CreateSurface(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Delete surface object.
    pub fn delete_surface(&self, handle: SurfaceHandle) {
        if self.surfaces
            .write()
            .unwrap()
            .dec_rc(handle.into())
            .is_some()
        {
            let task = PostFrameTask::DeleteSurface(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Lookup shader object from location.
    pub fn lookup_shader_from(&self, location: Location) -> Option<ShaderHandle> {
        self.shaders
            .read()
            .unwrap()
            .lookup(location)
            .map(|v| v.into())
    }

    /// Create a shader with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_shader(&self, location: Location, setup: ShaderSetup) -> Result<ShaderHandle> {
        if setup.uniform_variables.len() > MAX_UNIFORM_VARIABLES {
            return Err(Error::ShaderCreationFailure(format!(
                "Too many uniform variables (>= {:?}).",
                MAX_UNIFORM_VARIABLES
            )));
        }

        if setup.vs.is_empty() {
            return Err(Error::ShaderCreationFailure(
                "Vertex shader is required to describe a proper render pipeline.".into(),
            ));
        }

        if setup.fs.is_empty() {
            return Err(Error::ShaderCreationFailure(
                "Fragment shader is required to describe a proper render pipeline.".into(),
            ));
        }

        let handle = {
            let mut shaders = self.shaders.write().unwrap();
            if let Some(handle) = shaders.lookup(location) {
                shaders.inc_rc(handle);
                return Ok(handle.into());
            }

            let mut uniform_variable_names = HashMap::new();
            let mut uniform_variables = HashMap::new();
            for (name, v) in &setup.uniform_variables {
                let k: HashValue<str> = name.into();
                uniform_variables.insert(k, *v);
                uniform_variable_names.insert(k, name.clone());
            }

            let shader_state = AssetShaderState::ready(ShaderStateObject {
                render_state: setup.render_state,
                layout: setup.layout,
                uniform_variables: uniform_variables,
                uniform_variable_names: uniform_variable_names,
            });

            shaders.create(location, shader_state).into()
        };

        let task = PreFrameTask::CreatePipeline(handle, setup);
        self.frames.front().pre.push(task);
        Ok(handle)
    }

    /// Gets the shader state if exists.
    pub fn shader(&self, handle: ShaderHandle) -> Option<Arc<ShaderStateObject>> {
        self.shaders
            .read()
            .unwrap()
            .get(*handle)
            .and_then(|v| v.clone())
    }

    /// Returns true if shader is exists.
    pub fn is_shader_alive(&self, handle: ShaderHandle) -> bool {
        self.shaders.read().unwrap().is_alive(handle.into())
    }

    /// Delete shader state object.
    pub fn delete_shader(&self, handle: ShaderHandle) {
        if self.shaders
            .write()
            .unwrap()
            .dec_rc(handle.into())
            .is_some()
        {
            let task = PostFrameTask::DeletePipeline(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Lookup mesh object from location.
    pub fn lookup_mesh_from(&self, location: Location) -> Option<MeshHandle> {
        self.meshes
            .read()
            .unwrap()
            .lookup(location)
            .map(|v| v.into())
    }

    /// Create a new mesh object from location.
    pub fn create_mesh_from<T>(&self, location: Location, setup: MeshSetup) -> Result<MeshHandle>
    where
        T: MeshParser + Send + Sync + 'static,
    {
        if setup.hint != MeshHint::Immutable {
            return Err(Error::CreateMutableRemoteObject);
        }

        let (handle, state) = {
            let mut meshes = self.meshes.write().unwrap();
            if let Some(handle) = meshes.lookup(location) {
                meshes.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = Arc::new(RwLock::new(AssetState::NotReady));
            let handle = meshes.create(location, state.clone()).into();
            (handle, state)
        };

        let loader = MeshLoader::<T>::new(handle, state, setup, self.frames.clone());
        self.resource.load_async(loader, location.uri());
        Ok(handle)
    }

    /// Create a new mesh object.
    pub fn create_mesh<'a, 'b, T1, T2>(
        &self,
        location: Location,
        setup: MeshSetup,
        verts: T1,
        idxes: T2,
    ) -> Result<MeshHandle>
    where
        T1: Into<Option<&'a [u8]>>,
        T2: Into<Option<&'b [u8]>>,
    {
        if location.is_shared() {
            if setup.hint != MeshHint::Immutable {
                return Err(Error::CreateMutableSharedObject);
            }
        }

        let verts = verts.into();
        let idxes = idxes.into();

        if let Some(buf) = verts.as_ref() {
            if buf.len() > setup.vertex_buffer_len() {
                return Err(Error::OutOfBounds);
            }
        }

        if let Some(buf) = idxes.as_ref() {
            if buf.len() > setup.index_buffer_len() {
                return Err(Error::OutOfBounds);
            }
        }

        setup.validate()?;

        let handle = {
            let mut meshes = self.meshes.write().unwrap();
            if let Some(handle) = meshes.lookup(location) {
                meshes.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = Arc::new(RwLock::new(AssetState::ready(setup.clone())));
            meshes.create(location, state).into()
        };

        let mut frame = self.frames.front();
        let verts_ptr = verts.map(|v| frame.buf.extend_from_slice(v));
        let idxes_ptr = idxes.map(|v| frame.buf.extend_from_slice(v));
        let task = PreFrameTask::CreateMesh(handle, setup, verts_ptr, idxes_ptr);
        frame.pre.push(task);
        Ok(handle)
    }

    /// Gets the mesh state if exists.
    pub fn mesh(&self, handle: MeshHandle) -> Option<Arc<MeshStateObject>> {
        self.meshes
            .read()
            .unwrap()
            .get(*handle)
            .and_then(|v| v.read().unwrap().clone())
    }

    /// Returns true if shader is exists.
    pub fn is_mesh_alive(&self, handle: MeshHandle) -> bool {
        self.meshes.read().unwrap().is_alive(handle.into())
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(&self, mesh: MeshHandle, offset: usize, data: &[u8]) -> Result<()> {
        if let Some(state) = self.meshes.read().unwrap().get(mesh.into()) {
            if let AssetState::Ready(ref mso) = *state.read().unwrap() {
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
        if let Some(state) = self.meshes.read().unwrap().get(mesh.into()) {
            if let AssetState::Ready(ref mso) = *state.read().unwrap() {
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
        if self.meshes.write().unwrap().dec_rc(mesh.into()).is_some() {
            let task = PostFrameTask::DeleteMesh(mesh);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Lookup texture object from location.
    pub fn lookup_texture_from(&self, location: Location) -> Option<TextureHandle> {
        self.textures
            .read()
            .unwrap()
            .lookup(location)
            .map(|v| v.into())
    }

    /// Create texture object from location.
    pub fn create_texture_from<T>(
        &self,
        location: Location,
        setup: TextureSetup,
    ) -> Result<TextureHandle>
    where
        T: TextureParser + Send + Sync + 'static,
    {
        if setup.hint != TextureHint::Immutable {
            return Err(Error::CreateMutableRemoteObject);
        }

        let (handle, state) = {
            let mut textures = self.textures.write().unwrap();
            if let Some(handle) = textures.lookup(location) {
                textures.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = Arc::new(RwLock::new(AssetState::NotReady));
            let handle = textures.create(location, state.clone()).into();
            (handle, state)
        };

        let loader = TextureLoader::<T>::new(handle, state, setup, self.frames.clone());
        self.resource.load_async(loader, location.uri());
        Ok(handle)
    }

    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture<'a, T>(
        &self,
        location: Location,
        setup: TextureSetup,
        data: T,
    ) -> Result<TextureHandle>
    where
        T: Into<Option<&'a [u8]>>,
    {
        if location.is_shared() {
            if setup.hint != TextureHint::Immutable {
                return Err(Error::CreateMutableSharedObject);
            }
        }

        let handle = {
            let mut textures = self.textures.write().unwrap();
            if let Some(handle) = textures.lookup(location) {
                textures.inc_rc(handle);
                return Ok(handle.into());
            }

            let state = Arc::new(RwLock::new(AssetState::ready(setup)));
            textures.create(location, state).into()
        };

        let mut frame = self.frames.front();
        let ptr = data.into().map(|v| frame.buf.extend_from_slice(v));
        let task = PreFrameTask::CreateTexture(handle, setup, ptr);
        frame.pre.push(task);
        Ok(handle)
    }

    /// Gets the mesh state if exists.
    pub fn texture(&self, handle: TextureHandle) -> Option<Arc<TextureStateObject>> {
        self.textures
            .read()
            .unwrap()
            .get(*handle)
            .and_then(|v| v.read().unwrap().clone())
    }

    /// Returns true if texture is exists.
    pub fn texture_alive(&self, handle: TextureHandle) -> bool {
        self.textures.read().unwrap().is_alive(handle.into())
    }

    /// Update a contiguous subregion of an existing two-dimensional texture object.
    pub fn update_texture(&self, handle: TextureHandle, rect: Rect, data: &[u8]) -> Result<()> {
        if let Some(state) = self.textures.read().unwrap().get(handle.into()) {
            if let AssetState::Ready(ref texture) = *state.read().unwrap() {
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
        if self.textures
            .write()
            .unwrap()
            .dec_rc(handle.into())
            .is_some()
        {
            let task = PostFrameTask::DeleteTexture(handle);
            self.frames.front().post.push(task);
        }
    }
}

impl GraphicsSystemShared {
    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(&self, setup: RenderTextureSetup) -> Result<RenderTextureHandle> {
        let location = Location::unique("");
        let state = Arc::new(RwLock::new(AssetState::ready(setup)));
        let handle = self.render_textures
            .write()
            .unwrap()
            .create(location, state)
            .into();

        {
            let task = PreFrameTask::CreateRenderTexture(handle, setup);
            self.frames.front().pre.push(task);
        }

        Ok(handle)
    }

    /// Delete the render texture object.
    pub fn delete_render_texture(&self, handle: RenderTextureHandle) {
        if self.render_textures
            .write()
            .unwrap()
            .dec_rc(handle.into())
            .is_some()
        {
            let task = PostFrameTask::DeleteRenderTexture(handle);
            self.frames.front().post.push(task);
        }
    }
}
