use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crate::math::prelude::{Aabb2, Vector2};
use crate::prelude::CrResult;
use crate::res::utils::prelude::{ResourcePool, ResourceState};
use crate::utils::prelude::{DoubleBuf, ObjectPool};

use super::assets::mesh_loader::MeshLoader;
use super::assets::prelude::*;
use super::assets::texture_loader::TextureLoader;
use super::backends::frame::*;
use super::backends::{self, Visitor};
use super::errors::*;

/// The centralized management of video sub-system.
pub struct VideoSystem {
    lis: LifecycleListenerHandle,
    state: Arc<VideoState>,
}

struct VideoState {
    frames: Arc<DoubleBuf<Frame>>,
    surfaces: RwLock<ObjectPool<SurfaceHandle, SurfaceParams>>,
    shaders: RwLock<ObjectPool<ShaderHandle, ShaderParams>>,
    meshes: RwLock<ResourcePool<MeshHandle, MeshLoader>>,
    textures: RwLock<ResourcePool<TextureHandle, TextureLoader>>,
    render_textures: RwLock<ObjectPool<RenderTextureHandle, RenderTextureParams>>,
}

impl VideoState {
    fn new() -> Self {
        let frames = Arc::new(DoubleBuf::new(
            Frame::with_capacity(64 * 1024),
            Frame::with_capacity(64 * 1024),
        ));

        VideoState {
            surfaces: RwLock::new(ObjectPool::new()),
            shaders: RwLock::new(ObjectPool::new()),
            meshes: RwLock::new(ResourcePool::new(MeshLoader::new(frames.clone()))),
            textures: RwLock::new(ResourcePool::new(TextureLoader::new(frames.clone()))),
            render_textures: RwLock::new(ObjectPool::new()),
            frames,
        }
    }
}

struct Lifecycle {
    last_dimensions: Vector2<u32>,
    visitor: Box<dyn Visitor>,
    state: Arc<VideoState>,
}

impl LifecycleListener for Lifecycle {
    fn on_pre_update(&mut self) -> CrResult<()> {
        // Swap internal commands frame.
        self.state.frames.swap();
        self.state.frames.write().clear();
        self.state.meshes.write().unwrap().advance()?;
        self.state.textures.write().unwrap().advance()?;
        Ok(())
    }

    fn on_post_update(&mut self) -> CrResult<()> {
        let dimensions = dimensions_pixels();

        // Resize the window, which would recreate the underlying framebuffer.
        if dimensions != self.last_dimensions {
            self.last_dimensions = dimensions;
            crate::window::resize(dimensions);
        }

        self.state
            .frames
            .write_back_buf()
            .dispatch(self.visitor.as_mut(), self.last_dimensions)?;

        Ok(())
    }
}

impl Drop for VideoSystem {
    fn drop(&mut self) {
        crate::application::detach(self.lis);
    }
}

impl VideoSystem {
    /// Create a new `VideoSystem`.
    pub fn new() -> CrResult<Self> {
        let state = Arc::new(VideoState::new());
        let visitor = backends::new()?;

        Ok(VideoSystem {
            state: state.clone(),
            lis: crate::application::attach(Lifecycle {
                state,
                visitor,
                last_dimensions: dimensions_pixels(),
            }),
        })
    }

    /// Create a headless `VideoSystem`.
    pub fn headless() -> Self {
        let state = Arc::new(VideoState::new());
        let visitor = backends::new_headless();

        VideoSystem {
            state: state.clone(),
            lis: crate::application::attach(Lifecycle {
                state,
                visitor,
                last_dimensions: Vector2::new(0, 0),
            }),
        }
    }

    pub(crate) fn frames(&self) -> Arc<DoubleBuf<Frame>> {
        self.state.frames.clone()
    }
}

impl VideoSystem {
    /// Creates an surface with `SurfaceParams`.
    pub fn create_surface(&self, params: SurfaceParams) -> Result<SurfaceHandle> {
        let handle = self.state.surfaces.write().unwrap().create(params);

        {
            let cmd = Command::CreateSurface(Box::new((handle, params)));
            self.state.frames.write().cmds.push(cmd);
        }

        Ok(handle)
    }

    /// Gets the `SurfaceParams` if available.
    pub fn surface(&self, handle: SurfaceHandle) -> Option<SurfaceParams> {
        self.state.surfaces.read().unwrap().get(handle).cloned()
    }

    /// Get the resource state of specified surface.
    #[inline]
    pub fn surface_state(&self, handle: SurfaceHandle) -> ResourceState {
        if self.state.surfaces.read().unwrap().contains(handle) {
            ResourceState::Ok
        } else {
            ResourceState::NotReady
        }
    }

    /// Deletes surface object.
    pub fn delete_surface(&self, handle: SurfaceHandle) {
        if self.state.surfaces.write().unwrap().free(handle).is_some() {
            let cmd = Command::DeleteSurface(handle);
            self.state.frames.write().cmds.push(cmd);
        }
    }
}

impl VideoSystem {
    /// Create a shader with initial shaders and render state. It encapusulates all the
    /// informations we need to configurate graphics pipeline before real drawing.
    pub fn create_shader(
        &self,
        params: ShaderParams,
        vs: String,
        fs: String,
    ) -> Result<ShaderHandle> {
        params.validate(&vs, &fs)?;

        let handle = self.state.shaders.write().unwrap().create(params.clone());

        {
            let cmd = Command::CreateShader(Box::new((handle, params, vs, fs)));
            self.state.frames.write().cmds.push(cmd);
        }

        Ok(handle)
    }

    /// Gets the `ShaderParams` if available.
    #[inline]
    pub fn shader(&self, handle: ShaderHandle) -> Option<ShaderParams> {
        self.state.shaders.read().unwrap().get(handle).cloned()
    }

    /// Get the resource state of specified shader.
    #[inline]
    pub fn shader_state(&self, handle: ShaderHandle) -> ResourceState {
        if self.state.shaders.read().unwrap().contains(handle) {
            ResourceState::Ok
        } else {
            ResourceState::NotReady
        }
    }

    /// Delete shader state object.
    #[inline]
    pub fn delete_shader(&self, handle: ShaderHandle) {
        if self.state.shaders.write().unwrap().free(handle).is_some() {
            let cmd = Command::DeleteShader(handle);
            self.state.frames.write().cmds.push(cmd);
        }
    }
}

impl VideoSystem {
    /// Create a new mesh object.
    #[inline]
    pub fn create_mesh<T>(&self, params: MeshParams, data: T) -> CrResult<MeshHandle>
    where
        T: Into<Option<MeshData>>,
    {
        let mut meshes = self.state.meshes.write().unwrap();
        meshes.create((params, data.into()))
    }

    /// Creates a mesh object from file asynchronously.
    #[inline]
    pub fn create_mesh_from<T: AsRef<str>>(&self, url: T) -> CrResult<MeshHandle> {
        let mut meshes = self.state.meshes.write().unwrap();
        meshes.create_from(url)
    }

    /// Creates a mesh object from file asynchronously.
    #[inline]
    pub fn create_mesh_from_uuid(&self, uuid: Uuid) -> CrResult<MeshHandle> {
        let mut meshes = self.state.meshes.write().unwrap();
        meshes.create_from_uuid(uuid)
    }

    /// Gets the `MeshParams` if available.
    #[inline]
    pub fn mesh(&self, handle: MeshHandle) -> Option<MeshParams> {
        self.state.meshes.read().unwrap().resource(handle).cloned()
    }

    /// Get the resource state of specified mesh.
    #[inline]
    pub fn mesh_state(&self, handle: MeshHandle) -> ResourceState {
        self.state.meshes.read().unwrap().state(handle)
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(
        &self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> CrResult<()> {
        let meshes = self.state.meshes.read().unwrap();
        if meshes.contains(handle) {
            let mut frame = self.state.frames.write();
            let ptr = frame.bufs.extend_from_slice(data);
            let cmd = Command::UpdateVertexBuffer(handle, offset, ptr);
            frame.cmds.push(cmd);
            Ok(())
        } else {
            bail!("{:?} is invalid.", handle);
        }
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_index_buffer(
        &self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> CrResult<()> {
        let meshes = self.state.meshes.read().unwrap();
        if meshes.contains(handle) {
            let mut frame = self.state.frames.write();
            let ptr = frame.bufs.extend_from_slice(data);
            let cmd = Command::UpdateIndexBuffer(handle, offset, ptr);
            frame.cmds.push(cmd);
            Ok(())
        } else {
            bail!("{:?} is invalid.", handle);
        }
    }

    /// Delete mesh object.
    #[inline]
    pub fn delete_mesh(&self, handle: MeshHandle) {
        self.state.meshes.write().unwrap().delete(handle);
    }
}

impl VideoSystem {
    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture<T>(&self, params: TextureParams, data: T) -> CrResult<TextureHandle>
    where
        T: Into<Option<TextureData>>,
    {
        let mut textures = self.state.textures.write().unwrap();
        textures.create((params, data.into()))
    }

    /// Creates a texture object from file asynchronously.
    pub fn create_texture_from<T: AsRef<str>>(&self, url: T) -> CrResult<TextureHandle> {
        let mut textures = self.state.textures.write().unwrap();
        textures.create_from(url)
    }

    /// Creates a texture object from file asynchronously.
    pub fn create_texture_from_uuid(&self, uuid: Uuid) -> CrResult<TextureHandle> {
        let mut textures = self.state.textures.write().unwrap();
        textures.create_from_uuid(uuid)
    }

    /// Get the resource state of specified texture.
    #[inline]
    pub fn texture_state(&self, handle: TextureHandle) -> ResourceState {
        self.state.textures.read().unwrap().state(handle)
    }

    /// Update a contiguous subregion of an existing two-dimensional texture object.
    pub fn update_texture(
        &self,
        handle: TextureHandle,
        area: Aabb2<u32>,
        data: &[u8],
    ) -> CrResult<()> {
        let textures = self.state.textures.read().unwrap();
        if textures.contains(handle) {
            let mut frame = self.state.frames.write();
            let ptr = frame.bufs.extend_from_slice(data);
            let cmd = Command::UpdateTexture(handle, area, ptr);
            frame.cmds.push(cmd);
            Ok(())
        } else {
            bail!("{:?} is invalid.", handle);
        }
    }

    /// Delete the texture object.
    pub fn delete_texture(&self, handle: TextureHandle) {
        self.state.textures.write().unwrap().delete(handle);
    }
    /// Gets the `TextureParams` if available.
    #[inline]
    pub fn texture(&self,handle: TextureHandle)->Option<TextureParams>{
        self.state.textures.read().unwrap().resource(handle).cloned()
    }
}

impl VideoSystem {
    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(
        &self,
        params: RenderTextureParams,
    ) -> Result<RenderTextureHandle> {
        let handle = self.state.render_textures.write().unwrap().create(params);

        {
            let cmd = Command::CreateRenderTexture(Box::new((handle, params)));
            self.state.frames.write().cmds.push(cmd);
        }

        Ok(handle)
    }

    /// Gets the `RenderTextureParams` if available.
    pub fn render_texture(&self, handle: RenderTextureHandle) -> Option<RenderTextureParams> {
        self.state
            .render_textures
            .read()
            .unwrap()
            .get(handle)
            .cloned()
    }

    /// Get the resource state of specified render texture.
    #[inline]
    pub fn render_texture_state(&self, handle: RenderTextureHandle) -> ResourceState {
        if self.state.render_textures.read().unwrap().contains(handle) {
            ResourceState::Ok
        } else {
            ResourceState::NotReady
        }
    }

    /// Delete the render texture object.
    pub fn delete_render_texture(&self, handle: RenderTextureHandle) {
        if self
            .state
            .render_textures
            .write()
            .unwrap()
            .free(handle)
            .is_some()
        {
            let cmd = Command::DeleteRenderTexture(handle);
            self.state.frames.write().cmds.push(cmd);
        }
    }
}

fn dimensions_pixels() -> Vector2<u32> {
    let dimensions = crate::window::dimensions();
    let dpr = crate::window::device_pixel_ratio();
    Vector2::new(
        (dimensions.x as f32 * dpr) as u32,
        (dimensions.y as f32 * dpr) as u32,
    )
}
