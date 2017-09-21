use std::marker::PhantomData;
use std::borrow::Borrow;
use std::sync::MutexGuard;
use std::str;
use std::slice;
use std::mem;

use super::*;
use super::errors::*;
use super::resource::{ResourceHint, IndexFormat, VertexLayout, AttributeLayout,
                      FrameBufferAttachment};
use super::pipeline::{UniformVariable, Primitive};
use super::backend::Context;

#[derive(Debug, Clone, Copy)]
pub enum PreFrameTask {
    CreateView(ViewHandle, Option<FrameBufferHandle>),
    UpdateViewRect(ViewHandle, TaskBufferPtr<ViewRectDesc>),
    UpdateViewOrder(ViewHandle, u32),
    UpdateViewSequential(ViewHandle, bool),
    UpdateViewFrameBuffer(ViewHandle, Option<FrameBufferHandle>),

    CreatePipeline(PipelineStateHandle, TaskBufferPtr<PipelineDesc>),
    UpdatePipelineState(PipelineStateHandle, TaskBufferPtr<RenderState>),
    UpdatePipelineUniform(PipelineStateHandle, TaskBufferPtr<str>, TaskBufferPtr<UniformVariable>),

    CreateVertexBuffer(VertexBufferHandle, TaskBufferPtr<VertexBufferDesc>),
    UpdateVertexBuffer(VertexBufferHandle, u32, TaskBufferPtr<[u8]>),

    CreateIndexBuffer(IndexBufferHandle, TaskBufferPtr<IndexBufferDesc>),
    UpdateIndexBuffer(IndexBufferHandle, u32, TaskBufferPtr<[u8]>),

    CreateTexture(TextureHandle, TaskBufferPtr<TextureDesc>),
    CreateRenderTexture(TextureHandle, TaskBufferPtr<RenderTextureDesc>),
    UpdateTextureParameters(TextureHandle, TaskBufferPtr<TextureParametersDesc>),

    CreateRenderBuffer(RenderBufferHandle, TaskBufferPtr<RenderTextureDesc>),

    CreateFrameBuffer(FrameBufferHandle),
    UpdateFrameBufferAttachment(FrameBufferHandle, u32, FrameBufferAttachment),
    UpdateFrameBufferClear(FrameBufferHandle, TaskBufferPtr<FrameBufferClearDesc>),
}

#[derive(Debug, Clone, Copy)]
pub struct FrameTask {
    pub priority: u64,
    pub view: ViewHandle,
    pub pipeline: PipelineStateHandle,
    pub uniforms: TaskBufferPtr<[(TaskBufferPtr<str>, UniformVariable)]>,
    pub textures: TaskBufferPtr<[(TaskBufferPtr<str>, TextureHandle)]>,
    pub vb: VertexBufferHandle,
    pub ib: Option<IndexBufferHandle>,
    pub primitive: Primitive,
    pub from: u32,
    pub len: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum PostFrameTask {
    DeleteView(ViewHandle),
    DeletePipeline(PipelineStateHandle),
    DeleteVertexBuffer(VertexBufferHandle),
    DeleteIndexBuffer(IndexBufferHandle),
    DeleteTexture(TextureHandle),
    DeleteRenderBuffer(RenderBufferHandle),
    DeleteFrameBuffer(FrameBufferHandle),
}

#[derive(Debug, Clone, Copy)]
pub struct ViewRectDesc {
    pub position: (u16, u16),
    pub size: Option<(u16, u16)>,
}

#[derive(Debug, Clone, Copy)]
pub struct PipelineDesc {
    pub vs: TaskBufferPtr<str>,
    pub fs: TaskBufferPtr<str>,
    pub state: RenderState,
    pub attributes: AttributeLayout,
}

#[derive(Debug, Clone, Copy)]
pub struct VertexBufferDesc {
    pub layout: VertexLayout,
    pub hint: ResourceHint,
    pub size: u32,
    pub data: Option<TaskBufferPtr<[u8]>>,
}

#[derive(Debug, Clone, Copy)]
pub struct IndexBufferDesc {
    pub format: IndexFormat,
    pub hint: ResourceHint,
    pub size: u32,
    pub data: Option<TaskBufferPtr<[u8]>>,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureDesc {
    pub format: TextureFormat,
    pub address: TextureAddress,
    pub filter: TextureFilter,
    pub mipmap: bool,
    pub width: u32,
    pub height: u32,
    pub data: TaskBufferPtr<[u8]>,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderTextureDesc {
    pub format: RenderTextureFormat,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureParametersDesc {
    pub address: TextureAddress,
    pub filter: TextureFilter,
}

#[derive(Debug, Clone, Copy)]
pub struct FrameBufferClearDesc {
    pub clear_color: Option<Color>,
    pub clear_depth: Option<f32>,
    pub clear_stencil: Option<i32>,
}

pub struct Frame {
    pub pre: Vec<PreFrameTask>,
    pub drawcalls: Vec<FrameTask>,
    pub post: Vec<PostFrameTask>,
    pub buf: TaskBuffer,
}

impl Frame {
    /// Creates a new frame with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Frame {
            pre: Vec::new(),
            post: Vec::new(),
            drawcalls: Vec::new(),
            buf: TaskBuffer::with_capacity(capacity),
        }
    }

    pub unsafe fn clear(&mut self) {
        self.pre.clear();
        self.drawcalls.clear();
        self.post.clear();
        self.buf.clear();
    }

    pub unsafe fn dispatch(&mut self, context: &mut Context) -> Result<()> {
        let dimensions = context.dimensions().ok_or(ErrorKind::WindowNotExist)?;
        let mut device = &mut context.device();

        for v in &self.pre {
            match *v {
                PreFrameTask::CreateView(handle, framebuffer) => {
                    device.create_view(handle, framebuffer)?;
                }
                PreFrameTask::UpdateViewRect(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    device.update_view_rect(handle, desc.position, desc.size)?;
                }
                PreFrameTask::UpdateViewOrder(handle, priority) => {
                    device.update_view_order(handle, priority)?;
                }
                PreFrameTask::UpdateViewSequential(handle, seq) => {
                    device.update_view_sequential_mode(handle, seq)?;
                }
                PreFrameTask::UpdateViewFrameBuffer(handle, framebuffer) => {
                    device.update_view_framebuffer(handle, framebuffer)?;
                }
                PreFrameTask::CreatePipeline(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    device
                        .create_pipeline(handle,
                                         &desc.state,
                                         self.buf.as_str(desc.vs),
                                         self.buf.as_str(desc.fs),
                                         &desc.attributes)?;
                }
                PreFrameTask::UpdatePipelineState(handle, state) => {
                    let state = &self.buf.as_ref(state);
                    device.update_pipeline_state(handle, &state)?;
                }
                PreFrameTask::UpdatePipelineUniform(handle, name, variable) => {
                    let name = &self.buf.as_str(name);
                    let variable = &self.buf.as_ref(variable);
                    device.update_pipeline_uniform(handle, name, &variable)?;
                }
                PreFrameTask::CreateVertexBuffer(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    let data = desc.data.map(|ptr| self.buf.as_bytes(ptr));
                    device
                        .create_vertex_buffer(handle, &desc.layout, desc.hint, desc.size, data)?;
                }
                PreFrameTask::UpdateVertexBuffer(handle, offset, data) => {
                    let data = &self.buf.as_bytes(data);
                    device.update_vertex_buffer(handle, offset, &data)?;
                }
                PreFrameTask::CreateIndexBuffer(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    let data = desc.data.map(|ptr| self.buf.as_bytes(ptr));
                    device
                        .create_index_buffer(handle, desc.format, desc.hint, desc.size, data)?;
                }
                PreFrameTask::UpdateIndexBuffer(handle, offset, data) => {
                    let data = &self.buf.as_bytes(data);
                    device.update_index_buffer(handle, offset, &data)?;
                }
                PreFrameTask::CreateTexture(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    let data = &self.buf.as_bytes(desc.data);
                    device
                        .create_texture(handle,
                                        desc.format,
                                        desc.address,
                                        desc.filter,
                                        desc.mipmap,
                                        desc.width,
                                        desc.height,
                                        &data)?;
                }
                PreFrameTask::CreateRenderTexture(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    device
                        .create_render_texture(handle, desc.format, desc.width, desc.height)?;
                }
                PreFrameTask::UpdateTextureParameters(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    device
                        .update_texture_parameters(handle, desc.address, desc.filter)?;
                }
                PreFrameTask::CreateRenderBuffer(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    device
                        .create_render_buffer(handle, desc.format, desc.width, desc.height)?;
                }
                PreFrameTask::CreateFrameBuffer(handle) => {
                    device.create_framebuffer(handle)?;
                }
                PreFrameTask::UpdateFrameBufferAttachment(handle, slot, attachment) => {
                    match attachment {
                        FrameBufferAttachment::RenderBuffer(rb) => {
                            device
                                .update_framebuffer_with_renderbuffer(handle, rb, slot)?;
                        }
                        FrameBufferAttachment::Texture(texture) => {
                            device
                                .update_framebuffer_with_texture(handle, texture, slot)?;
                        }
                    };
                }
                PreFrameTask::UpdateFrameBufferClear(handle, desc) => {
                    let desc = &self.buf.as_ref(desc);
                    device
                        .update_framebuffer_clear(handle,
                                                  desc.clear_color,
                                                  desc.clear_depth,
                                                  desc.clear_stencil)?;
                }
            }
        }

        for dc in &self.drawcalls {
            device
                .submit(dc.priority,
                        dc.view,
                        dc.pipeline,
                        dc.textures,
                        dc.uniforms,
                        dc.vb,
                        dc.ib,
                        dc.primitive,
                        dc.from,
                        dc.len)?;
        }
        device.flush(&self.buf, dimensions)?;

        for v in &self.post {
            match *v {
                PostFrameTask::DeleteView(handle) => {
                    device.delete_view(handle)?;
                }
                PostFrameTask::DeletePipeline(handle) => {
                    device.delete_pipeline(handle)?;
                }
                PostFrameTask::DeleteVertexBuffer(handle) => {
                    device.delete_vertex_buffer(handle)?;
                }
                PostFrameTask::DeleteIndexBuffer(handle) => {
                    device.delete_index_buffer(handle)?;
                }
                PostFrameTask::DeleteTexture(handle) => {
                    device.delete_texture(handle)?;
                }
                PostFrameTask::DeleteRenderBuffer(handle) => {
                    device.delete_render_buffer(handle)?;
                }
                PostFrameTask::DeleteFrameBuffer(handle) => {
                    device.delete_framebuffer(handle)?;
                }
            }
        }

        Ok(())
    }
}

/// A frame task builder.
pub struct FrameTaskBuilder<'a> {
    frame: MutexGuard<'a, Frame>,

    order: u64,
    uniforms: Vec<(TaskBufferPtr<str>, UniformVariable)>,
    textures: Vec<(TaskBufferPtr<str>, TextureHandle)>,

    vb: Option<VertexBufferHandle>,
    ib: Option<IndexBufferHandle>,
    view: Option<ViewHandle>,
    pso: Option<PipelineStateHandle>,
}

impl<'a> FrameTaskBuilder<'a> {
    pub fn new<'b>(frame: MutexGuard<'b, Frame>) -> Self
        where 'b: 'a
    {
        FrameTaskBuilder {
            frame: frame,
            order: 0,
            view: None,
            pso: None,
            uniforms: Vec::new(),
            textures: Vec::new(),
            vb: None,
            ib: None,
        }
    }

    pub fn with_order(&mut self, order: u64) -> &mut Self {
        self.order = order;
        self
    }

    pub fn with_view(&mut self, view: ViewHandle) -> &mut Self {
        self.view = Some(view);
        self
    }

    pub fn with_pipeline(&mut self, pso: PipelineStateHandle) -> &mut Self {
        self.pso = Some(pso);
        self
    }

    pub fn with_data(&mut self,
                     vb: VertexBufferHandle,
                     ib: Option<IndexBufferHandle>)
                     -> &mut Self {
        self.vb = Some(vb);
        self.ib = ib;
        self
    }

    pub fn with_uniform_variable(&mut self, field: &str, variable: UniformVariable) -> &mut Self {
        let field = self.frame.buf.extend_from_str(field);
        self.uniforms.push((field, variable));
        self
    }

    pub fn with_texture(&mut self, field: &str, texture: TextureHandle) -> &mut Self {
        let field = self.frame.buf.extend_from_str(field);
        self.textures.push((field, texture));
        self
    }

    pub fn submit(&mut self, primitive: Primitive, from: u32, len: u32) -> Result<()> {
        let view = self.view.ok_or(ErrorKind::CanNotDrawWithoutView)?;
        let pso = self.pso.ok_or(ErrorKind::CanNotDrawWithoutPipelineState)?;
        let vb = self.vb.ok_or(ErrorKind::CanNotDrawWihtoutVertexBuffer)?;

        let uniforms = self.frame.buf.extend_from_slice(self.uniforms.as_slice());
        let textures = self.frame.buf.extend_from_slice(self.textures.as_slice());

        let task = FrameTask {
            priority: self.order,
            view: view,
            pipeline: pso,
            textures: textures,
            uniforms: uniforms,
            vb: vb,
            ib: self.ib,
            primitive: primitive,
            from: from,
            len: len,
        };

        self.frame.drawcalls.push(task);
        Ok(())
    }
}

/// Where we store all the intermediate bytes.
pub struct TaskBuffer(Vec<u8>);

impl TaskBuffer {
    /// Creates a new task buffer with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        TaskBuffer(Vec::with_capacity(capacity))
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn extend<T>(&mut self, value: &T) -> TaskBufferPtr<T>
        where T: Copy
    {
        let data =
            unsafe { slice::from_raw_parts(value as *const T as *const u8, mem::size_of::<T>()) };

        self.0.extend_from_slice(data);
        TaskBufferPtr {
            position: (self.0.len() - data.len()) as u32,
            size: data.len() as u32,
            _phantom: PhantomData,
        }
    }

    /// Clones and appends all elements in a slice to the buffer.
    pub fn extend_from_slice<T>(&mut self, slice: &[T]) -> TaskBufferPtr<[T]>
        where T: Copy
    {
        let len = mem::size_of::<T>().wrapping_mul(slice.len());
        let u8_slice = unsafe { slice::from_raw_parts(slice.as_ptr() as *const u8, len) };
        self.0.extend_from_slice(u8_slice);
        TaskBufferPtr {
            position: (self.0.len() - len) as u32,
            size: len as u32,
            _phantom: PhantomData,
        }
    }

    /// Clones and append all bytes in a string slice to the buffer.
    pub fn extend_from_str<T>(&mut self, value: T) -> TaskBufferPtr<str>
        where T: Borrow<str>
    {
        let slice = self.extend_from_slice(value.borrow().as_bytes());
        TaskBufferPtr {
            position: slice.position,
            size: slice.size,
            _phantom: PhantomData,
        }
    }

    /// Returns reference to object indicated by `TaskBufferPtr`.
    #[inline]
    pub fn as_ref<T>(&self, ptr: TaskBufferPtr<T>) -> &T
        where T: Copy
    {
        let slice = self.as_bytes(ptr);
        assert_eq!(slice.len(), mem::size_of::<T>());
        unsafe { &*(slice.as_ptr() as *const _) }
    }

    /// Returns a object slice indicated by `TaskBufferPtr.
    #[inline]
    pub fn as_slice<T>(&self, ptr: TaskBufferPtr<[T]>) -> &[T]
        where T: Copy
    {
        let slice = self.as_bytes(ptr);
        let len = slice.len() / mem::size_of::<T>();
        assert_eq!(slice.len(), mem::size_of::<T>().wrapping_mul(len));
        unsafe { slice::from_raw_parts(slice.as_ptr() as *const T, len) }
    }

    /// Returns string slice indicated by `TaskBufferPtr`.
    #[inline]
    pub fn as_str(&self, ptr: TaskBufferPtr<str>) -> &str {
        str::from_utf8(self.as_bytes(ptr)).unwrap()
    }

    #[inline]
    pub fn as_bytes<T>(&self, slice: TaskBufferPtr<T>) -> &[u8]
        where T: ?Sized
    {
        &self.0[slice.position as usize..(slice.position + slice.size) as usize]
    }
}

/// A view into our `DataBuffer`, indicates where the object `T` stored.
#[derive(Debug)]
pub struct TaskBufferPtr<T>
    where T: ?Sized
{
    position: u32,
    size: u32,
    _phantom: PhantomData<T>,
}

impl<T> Clone for TaskBufferPtr<T>
    where T: ?Sized
{
    fn clone(&self) -> Self {
        TaskBufferPtr {
            position: self.position,
            size: self.size,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for TaskBufferPtr<T> where T: ?Sized {}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    struct UpdateViewRect {
        position: (u16, u16),
        size: (u16, u16),
    }

    #[test]
    fn buf() {
        let mut buffer = TaskBuffer::with_capacity(128);

        let mut uvp = UpdateViewRect::default();
        uvp.position = (256, 128);
        let slice_uvp = buffer.extend(&uvp);

        let int = 128 as u32;
        let slice_int = buffer.extend(&int);

        assert_eq!(*buffer.as_ref(slice_int), int);
        assert_eq!(*buffer.as_ref(slice_uvp), uvp);

        let arr = [1, 2, 3];
        let slice_arr = buffer.extend(&arr);
        assert_eq!(*buffer.as_ref(slice_arr), arr);

        let slice_arr_1_2 = buffer.extend_from_slice(&arr[0..2]);
        assert_eq!(buffer.as_slice(slice_arr_1_2), &arr[0..2]);

        let text = "string serialization";
        let slice_text = buffer.extend_from_str(text);
        assert_eq!(text, buffer.as_str(slice_text));
    }
}