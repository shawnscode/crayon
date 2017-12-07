use super::{TextureHandle, VertexBufferHandle, IndexBufferHandle, ViewStateHandle,
            PipelineStateHandle, FrameBufferHandle, RenderBufferHandle};
use super::GraphicsSystemShared;

pub enum GraphicsResource {
    Texture(TextureHandle),
    VertexBuffer(VertexBufferHandle),
    IndexBuffer(IndexBufferHandle),
    ViewState(ViewStateHandle),
    PipelineState(PipelineStateHandle),
    FrameBuffer(FrameBufferHandle),
    RenderBuffer(RenderBufferHandle),
}

impl From<TextureHandle> for GraphicsResource {
    fn from(handle: TextureHandle) -> GraphicsResource {
        GraphicsResource::Texture(handle)
    }
}

impl From<VertexBufferHandle> for GraphicsResource {
    fn from(handle: VertexBufferHandle) -> GraphicsResource {
        GraphicsResource::VertexBuffer(handle)
    }
}

impl From<IndexBufferHandle> for GraphicsResource {
    fn from(handle: IndexBufferHandle) -> GraphicsResource {
        GraphicsResource::IndexBuffer(handle)
    }
}

impl From<ViewStateHandle> for GraphicsResource {
    fn from(handle: ViewStateHandle) -> GraphicsResource {
        GraphicsResource::ViewState(handle)
    }
}

impl From<PipelineStateHandle> for GraphicsResource {
    fn from(handle: PipelineStateHandle) -> GraphicsResource {
        GraphicsResource::PipelineState(handle)
    }
}

impl From<FrameBufferHandle> for GraphicsResource {
    fn from(handle: FrameBufferHandle) -> GraphicsResource {
        GraphicsResource::FrameBuffer(handle)
    }
}

impl From<RenderBufferHandle> for GraphicsResource {
    fn from(handle: RenderBufferHandle) -> GraphicsResource {
        GraphicsResource::RenderBuffer(handle)
    }
}

pub struct GraphicsResourceLabel {
    stack: Vec<GraphicsResource>,
}

impl GraphicsResourceLabel {
    pub fn new() -> Self {
        GraphicsResourceLabel { stack: Vec::new() }
    }

    pub fn push<T>(&mut self, resource: T) -> T
        where T: Copy + Into<GraphicsResource>
    {
        self.stack.push(resource.into());
        resource
    }

    pub fn clear(&mut self, video: &GraphicsSystemShared) {
        for v in self.stack.drain(..) {
            match v {
                GraphicsResource::Texture(handle) => video.delete_texture(handle),
                GraphicsResource::VertexBuffer(handle) => video.delete_vertex_buffer(handle),
                GraphicsResource::IndexBuffer(handle) => video.delete_index_buffer(handle),
                GraphicsResource::ViewState(handle) => video.delete_view(handle),
                GraphicsResource::PipelineState(handle) => video.delete_pipeline(handle),
                GraphicsResource::FrameBuffer(handle) => video.delete_framebuffer(handle),
                GraphicsResource::RenderBuffer(handle) => video.delete_render_buffer(handle),
            }
        }
    }
}