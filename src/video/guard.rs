use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use video::assets::mesh_loader::MeshParser;
use video::assets::prelude::*;
use video::assets::texture_loader::TextureParser;
use video::errors::Result;
use video::VideoSystemShared;

pub struct VideoSystemGuard {
    stack: HashMap<Resource, u32>,
    video: Arc<VideoSystemShared>,
}

impl Deref for VideoSystemGuard {
    type Target = VideoSystemShared;

    fn deref(&self) -> &Self::Target {
        &self.video
    }
}

impl VideoSystemGuard {
    pub fn new(video: Arc<VideoSystemShared>) -> Self {
        VideoSystemGuard {
            stack: HashMap::new(),
            video: video,
        }
    }

    #[inline]
    pub fn create_surface(&mut self, setup: SurfaceSetup) -> Result<SurfaceHandle> {
        let v = self.video.create_surface(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn delete_surface(&mut self, handle: SurfaceHandle) {
        self.pop(handle);
    }

    #[inline]
    pub fn create_shader(&mut self, setup: ShaderSetup) -> Result<ShaderHandle> {
        let v = self.video.create_shader(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn delete_shader(&mut self, handle: ShaderHandle) {
        self.pop(handle);
    }

    #[inline]
    pub fn create_mesh_from_file<T>(&mut self, setup: MeshSetup) -> Result<MeshHandle>
    where
        T: MeshParser + Send + Sync + 'static,
    {
        let v = self.video.create_mesh_from_file::<T>(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn create_mesh(&mut self, setup: MeshSetup) -> Result<MeshHandle> {
        let v = self.video.create_mesh(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn delete_mesh(&mut self, handle: MeshHandle) {
        self.pop(handle);
    }

    #[inline]
    pub fn create_texture_from_file<T>(&mut self, setup: TextureSetup) -> Result<TextureHandle>
    where
        T: TextureParser + Send + Sync + 'static,
    {
        let v = self.video.create_texture_from_file::<T>(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn create_texture(&mut self, setup: TextureSetup) -> Result<TextureHandle> {
        let v = self.video.create_texture(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn delete_texture(&mut self, handle: TextureHandle) {
        self.pop(handle);
    }

    #[inline]
    pub fn create_render_texture(
        &mut self,
        setup: RenderTextureSetup,
    ) -> Result<RenderTextureHandle> {
        let v = self.video.create_render_texture(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn delete_render_texture(&mut self, handle: RenderTextureHandle) {
        self.pop(handle);
    }

    fn pop<T>(&mut self, resource: T)
    where
        T: Copy + Into<Resource>,
    {
        let resource = resource.into();
        let delete = if let Some(v) = self.stack.get_mut(&resource) {
            *v -= 1;
            *v <= 0
        } else {
            panic!("Trying to delete resource that do not belongs to this `VideoSystemGuard`.");
        };

        if delete {
            self.stack.remove(&resource);
            Self::delete(&self.video, resource);
        }
    }

    fn push<T>(&mut self, resource: T) -> T
    where
        T: Copy + Into<Resource>,
    {
        *self.stack.entry(resource.into()).or_insert(0) += 1;
        resource
    }

    fn delete(video: &VideoSystemShared, handle: Resource) {
        match handle {
            Resource::Texture(handle) => video.delete_texture(handle),
            Resource::RenderTexture(handle) => video.delete_render_texture(handle),
            Resource::Mesh(handle) => video.delete_mesh(handle),
            Resource::Surface(handle) => video.delete_surface(handle),
            Resource::ShaderState(handle) => video.delete_shader(handle),
        }
    }
}

impl Drop for VideoSystemGuard {
    fn drop(&mut self) {
        for v in self.stack.keys() {
            Self::delete(&self.video, *v);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Resource {
    Texture(TextureHandle),
    RenderTexture(RenderTextureHandle),
    Mesh(MeshHandle),
    Surface(SurfaceHandle),
    ShaderState(ShaderHandle),
}

impl From<TextureHandle> for Resource {
    fn from(handle: TextureHandle) -> Resource {
        Resource::Texture(handle)
    }
}

impl From<RenderTextureHandle> for Resource {
    fn from(handle: RenderTextureHandle) -> Resource {
        Resource::RenderTexture(handle)
    }
}

impl From<MeshHandle> for Resource {
    fn from(handle: MeshHandle) -> Resource {
        Resource::Mesh(handle)
    }
}

impl From<SurfaceHandle> for Resource {
    fn from(handle: SurfaceHandle) -> Resource {
        Resource::Surface(handle)
    }
}

impl From<ShaderHandle> for Resource {
    fn from(handle: ShaderHandle) -> Resource {
        Resource::ShaderState(handle)
    }
}
