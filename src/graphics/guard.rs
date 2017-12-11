use std::sync::Arc;

use resource::Location;
use super::*;
use super::errors::*;
use super::assets::texture_loader::TextureParser;

pub struct RAIIGuard {
    stack: Vec<Resource>,
    video: Arc<GraphicsSystemShared>,
}

impl RAIIGuard {
    pub fn new(video: Arc<GraphicsSystemShared>) -> Self {
        RAIIGuard {
            stack: Vec::new(),
            video: video,
        }
    }

    #[inline(always)]
    pub fn create_view(&mut self, setup: SurfaceSetup) -> Result<SurfaceHandle> {
        let v = self.video.create_view(setup)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_shader(&mut self, setup: ShaderSetup) -> Result<ShaderHandle> {
        let v = self.video.create_shader(setup)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_framebuffer(&mut self, setup: FrameBufferSetup) -> Result<FrameBufferHandle> {
        let v = self.video.create_framebuffer(setup)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_render_buffer(&mut self, setup: RenderBufferSetup) -> Result<RenderBufferHandle> {
        let v = self.video.create_render_buffer(setup)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_vertex_buffer(&mut self,
                                setup: VertexBufferSetup,
                                data: Option<&[u8]>)
                                -> Result<VertexBufferHandle> {
        let v = self.video.create_vertex_buffer(setup, data)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_index_buffer(&mut self,
                               setup: IndexBufferSetup,
                               data: Option<&[u8]>)
                               -> Result<IndexBufferHandle> {
        let v = self.video.create_index_buffer(setup, data)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_texture_from<T>(&mut self,
                                  location: Location,
                                  setup: TextureSetup)
                                  -> Result<TextureHandle>
        where T: TextureParser + Send + Sync + 'static
    {
        let v = self.video.create_texture_from::<T>(location, setup)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_render_texture(&mut self, setup: RenderTextureSetup) -> Result<TextureHandle> {
        let v = self.video.create_render_texture(setup)?;
        Ok(self.push(v))
    }

    #[inline(always)]
    pub fn create_texture(&mut self,
                          setup: TextureSetup,
                          data: Option<&[u8]>)
                          -> Result<TextureHandle> {
        let v = self.video.create_texture(setup, data)?;
        Ok(self.push(v))
    }

    pub fn clear(&mut self) {
        for v in self.stack.drain(..) {
            match v {
                Resource::Texture(handle) => self.video.delete_texture(handle),
                Resource::VertexBuffer(handle) => self.video.delete_vertex_buffer(handle),
                Resource::IndexBuffer(handle) => self.video.delete_index_buffer(handle),
                Resource::Surface(handle) => self.video.delete_view(handle),
                Resource::PipelineState(handle) => self.video.delete_shader(handle),
                Resource::FrameBuffer(handle) => self.video.delete_framebuffer(handle),
                Resource::RenderBuffer(handle) => self.video.delete_render_buffer(handle),
            }
        }
    }

    fn push<T>(&mut self, resource: T) -> T
        where T: Copy + Into<Resource>
    {
        self.stack.push(resource.into());
        resource
    }
}

impl Drop for RAIIGuard {
    fn drop(&mut self) {
        self.clear();
    }
}

enum Resource {
    Texture(TextureHandle),
    VertexBuffer(VertexBufferHandle),
    IndexBuffer(IndexBufferHandle),
    Surface(SurfaceHandle),
    PipelineState(ShaderHandle),
    FrameBuffer(FrameBufferHandle),
    RenderBuffer(RenderBufferHandle),
}

impl From<TextureHandle> for Resource {
    fn from(handle: TextureHandle) -> Resource {
        Resource::Texture(handle)
    }
}

impl From<VertexBufferHandle> for Resource {
    fn from(handle: VertexBufferHandle) -> Resource {
        Resource::VertexBuffer(handle)
    }
}

impl From<IndexBufferHandle> for Resource {
    fn from(handle: IndexBufferHandle) -> Resource {
        Resource::IndexBuffer(handle)
    }
}

impl From<SurfaceHandle> for Resource {
    fn from(handle: SurfaceHandle) -> Resource {
        Resource::Surface(handle)
    }
}

impl From<ShaderHandle> for Resource {
    fn from(handle: ShaderHandle) -> Resource {
        Resource::PipelineState(handle)
    }
}

impl From<FrameBufferHandle> for Resource {
    fn from(handle: FrameBufferHandle) -> Resource {
        Resource::FrameBuffer(handle)
    }
}

impl From<RenderBufferHandle> for Resource {
    fn from(handle: RenderBufferHandle) -> Resource {
        Resource::RenderBuffer(handle)
    }
}
