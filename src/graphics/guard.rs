use std::sync::Arc;
use std::ops::Deref;

use graphics::GraphicsSystemShared;
use graphics::errors::Result;
use graphics::assets::prelude::*;
use graphics::assets::texture_loader::TextureParser;
use graphics::assets::mesh_loader::MeshParser;

pub struct GraphicsSystemGuard {
    stack: Vec<Resource>,
    video: Arc<GraphicsSystemShared>,
}

impl Deref for GraphicsSystemGuard {
    type Target = GraphicsSystemShared;

    fn deref(&self) -> &Self::Target {
        &self.video
    }
}

impl GraphicsSystemGuard {
    pub fn new(video: Arc<GraphicsSystemShared>) -> Self {
        GraphicsSystemGuard {
            stack: Vec::new(),
            video: video,
        }
    }

    #[inline]
    pub fn create_surface(&mut self, setup: SurfaceSetup) -> Result<SurfaceHandle> {
        let v = self.video.create_surface(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn create_shader(&mut self, setup: ShaderSetup) -> Result<ShaderHandle> {
        let v = self.video.create_shader(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn create_mesh_from<T>(&mut self, setup: MeshSetup) -> Result<MeshHandle>
    where
        T: MeshParser + Send + Sync + 'static,
    {
        let v = self.video.create_mesh_from::<T>(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn create_mesh(&mut self, setup: MeshSetup) -> Result<MeshHandle> {
        let v = self.video.create_mesh(setup)?;
        Ok(self.push(v))
    }

    #[inline]
    pub fn create_texture_from<T>(&mut self, setup: TextureSetup) -> Result<TextureHandle>
    where
        T: TextureParser + Send + Sync + 'static,
    {
        let v = self.video.create_texture_from::<T>(setup)?;
        Ok(self.push(v))
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
    pub fn create_texture(&mut self, setup: TextureSetup) -> Result<TextureHandle> {
        let v = self.video.create_texture(setup)?;
        Ok(self.push(v))
    }

    pub fn clear(&mut self) {
        for v in self.stack.drain(..) {
            match v {
                Resource::Texture(handle) => self.video.delete_texture(handle),
                Resource::RenderTexture(handle) => self.video.delete_render_texture(handle),
                Resource::Mesh(handle) => self.video.delete_mesh(handle),
                Resource::Surface(handle) => self.video.delete_surface(handle),
                Resource::ShaderState(handle) => self.video.delete_shader(handle),
            }
        }
    }

    fn push<T>(&mut self, resource: T) -> T
    where
        T: Copy + Into<Resource>,
    {
        self.stack.push(resource.into());
        resource
    }
}

impl Drop for GraphicsSystemGuard {
    fn drop(&mut self) {
        self.clear();
    }
}

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
