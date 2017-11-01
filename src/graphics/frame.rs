use std::marker::PhantomData;
use std::borrow::Borrow;
use std::sync::MutexGuard;
use std::collections::HashMap;
use std::str;
use std::slice;
use std::mem;

use super::*;
use super::errors::*;
use super::backend::Device;

use utils;

#[derive(Debug, Clone)]
pub enum PreFrameTask {
    CreateView(ViewStateHandle, ViewStateSetup),
    CreatePipeline(PipelineStateHandle, PipelineStateSetup, String, String),
    CreateFrameBuffer(FrameBufferHandle, FrameBufferSetup),
    CreateTexture(TextureHandle, TextureSetup, Option<TaskBufferPtr<[u8]>>),
    UpdateTexture(TextureHandle, Rect, TaskBufferPtr<[u8]>),
    CreateRenderTexture(TextureHandle, RenderTextureSetup),
    CreateRenderBuffer(RenderBufferHandle, RenderBufferSetup),
    CreateVertexBuffer(VertexBufferHandle, VertexBufferSetup, Option<TaskBufferPtr<[u8]>>),
    UpdateVertexBuffer(VertexBufferHandle, usize, TaskBufferPtr<[u8]>),
    CreateIndexBuffer(IndexBufferHandle, IndexBufferSetup, Option<TaskBufferPtr<[u8]>>),
    UpdateIndexBuffer(IndexBufferHandle, usize, TaskBufferPtr<[u8]>),
}

#[derive(Debug, Clone, Copy)]
pub enum PostFrameTask {
    DeleteView(ViewStateHandle),
    DeletePipeline(PipelineStateHandle),
    DeleteVertexBuffer(VertexBufferHandle),
    DeleteIndexBuffer(IndexBufferHandle),
    DeleteTexture(TextureHandle),
    DeleteRenderBuffer(RenderBufferHandle),
    DeleteFrameBuffer(FrameBufferHandle),
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub pre: Vec<PreFrameTask>,
    pub drawcalls: Vec<DrawCall>,
    pub post: Vec<PostFrameTask>,
    pub buf: TaskBuffer,
}

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

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

    /// Clear the frame, removing all data.
    pub unsafe fn clear(&mut self) {
        self.pre.clear();
        self.drawcalls.clear();
        self.post.clear();
        self.buf.clear();
    }

    /// Dispatch frame tasks and draw calls to the backend context.
    pub unsafe fn dispatch(&mut self, device: &mut Device, dimensions: (u32, u32)) -> Result<()> {
        for v in self.pre.drain(..) {
            match v {
                PreFrameTask::CreateView(handle, setup) => {
                    device.create_view(handle, setup)?;
                }
                PreFrameTask::CreatePipeline(handle, setup, vs, fs) => {
                    device.create_pipeline(handle, setup, vs, fs)?;
                }
                PreFrameTask::CreateVertexBuffer(handle, setup, data) => {
                    let field = &self.buf;
                    let buf = data.map(|v| field.as_slice(v));
                    device.create_vertex_buffer(handle, setup, buf)?;
                }
                PreFrameTask::UpdateVertexBuffer(handle, offset, data) => {
                    let data = &self.buf.as_bytes(data);
                    device.update_vertex_buffer(handle, offset, &data)?;
                }
                PreFrameTask::CreateIndexBuffer(handle, setup, data) => {
                    let field = &self.buf;
                    let buf = data.map(|v| field.as_slice(v));
                    device.create_index_buffer(handle, setup, buf)?;
                }
                PreFrameTask::UpdateIndexBuffer(handle, offset, data) => {
                    let buf = &self.buf.as_bytes(data);
                    device.update_index_buffer(handle, offset, &buf)?;
                }
                PreFrameTask::CreateTexture(handle, setup, data) => {
                    let field = &self.buf;
                    let buf = data.map(|v| field.as_slice(v));
                    device.create_texture(handle, setup, buf)?;
                }
                PreFrameTask::UpdateTexture(handle, rect, data) => {
                    let buf = &self.buf.as_bytes(data);
                    device.update_texture(handle, rect, &buf)?;
                }
                PreFrameTask::CreateRenderTexture(handle, setup) => {
                    device.create_render_texture(handle, setup)?;
                }
                PreFrameTask::CreateRenderBuffer(handle, setup) => {
                    device.create_render_buffer(handle, setup)?;
                }
                PreFrameTask::CreateFrameBuffer(handle, setup) => {
                    device.create_framebuffer(handle)?;

                    // Update framebuffer's attachments.
                    for (i, attachment) in setup.attachments().iter().enumerate() {
                        if let &Some(v) = attachment {
                            let i = i as u32;
                            match v {
                                FrameBufferAttachment::RenderBuffer(rb) => {
                                    device.update_framebuffer_with_renderbuffer(handle, rb, i)?;
                                }
                                FrameBufferAttachment::Texture(texture) => {
                                    device.update_framebuffer_with_texture(handle, texture, i)?;
                                }
                            };
                        }
                    }
                }
            }
        }

        for dc in &self.drawcalls {
            device
                .submit(dc.order,
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

#[derive(Debug, Clone, Copy)]
pub struct DrawCall {
    order: u64,
    view: ViewStateHandle,
    pipeline: PipelineStateHandle,
    uniforms: TaskBufferPtr<[(TaskBufferPtr<str>, UniformVariable)]>,
    textures: TaskBufferPtr<[(TaskBufferPtr<str>, TextureHandle)]>,
    vb: VertexBufferHandle,
    ib: Option<IndexBufferHandle>,
    primitive: Primitive,
    from: u32,
    len: u32,
}

/// A draw call builder.
pub struct DrawCallBuilder<'a> {
    frame: MutexGuard<'a, Frame>,

    order: u64,
    uniforms: Vec<(TaskBufferPtr<str>, UniformVariable)>,
    textures: Vec<(TaskBufferPtr<str>, TextureHandle)>,

    vb: Option<VertexBufferHandle>,
    ib: Option<IndexBufferHandle>,
    view: Option<ViewStateHandle>,
    pso: Option<PipelineStateHandle>,
}

impl<'a> DrawCallBuilder<'a> {
    /// Create a new ane empty draw call builder.
    pub fn new<'b>(frame: MutexGuard<'b, Frame>) -> Self
        where 'b: 'a
    {
        DrawCallBuilder {
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

    /// Set the sorting order.
    pub fn with_order(&mut self, order: u64) -> &mut Self {
        self.order = order;
        self
    }

    /// Bind the handle of `ViewStateObject`.
    pub fn with_view(&mut self, view: ViewStateHandle) -> &mut Self {
        self.view = Some(view);
        self
    }

    /// Bind the handle of `PipelineStateObject`.
    pub fn with_pipeline(&mut self, pso: PipelineStateHandle) -> &mut Self {
        self.pso = Some(pso);
        self
    }

    /// Bind vertex buffer and optional index buffer.
    pub fn with_data(&mut self,
                     vb: VertexBufferHandle,
                     ib: Option<IndexBufferHandle>)
                     -> &mut Self {
        self.vb = Some(vb);
        self.ib = ib;
        self
    }

    /// Bind the named field with `UniformVariable`.
    pub fn with_uniform_variable(&mut self, field: &str, variable: UniformVariable) -> &mut Self {
        let field = self.frame.buf.extend_from_str(field);
        self.uniforms.push((field, variable));
        self
    }

    /// Bind the field with texture.
    pub fn with_texture(&mut self, field: &str, texture: TextureHandle) -> &mut Self {
        let field = self.frame.buf.extend_from_str(field);
        self.textures.push((field, texture));
        self
    }

    /// Submit primitive for drawing, within view all draw commands are executed after
    /// resource manipulation, such like `create_vertex_buffer`, `update_vertex_buffer`,
    /// etc.
    pub fn submit(&mut self, primitive: Primitive, from: u32, len: u32) -> Result<()> {
        let view = self.view.ok_or(ErrorKind::CanNotDrawWithoutView)?;
        let pso = self.pso.ok_or(ErrorKind::CanNotDrawWithoutPipelineState)?;
        let vb = self.vb.ok_or(ErrorKind::CanNotDrawWihtoutVertexBuffer)?;

        let uniforms = self.frame.buf.extend_from_slice(self.uniforms.as_slice());
        let textures = self.frame.buf.extend_from_slice(self.textures.as_slice());

        let task = DrawCall {
            order: self.order,
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
#[derive(Debug, Clone)]
pub struct TaskBuffer(Vec<u8>, HashMap<u64, TaskBufferPtr<str>>);

impl TaskBuffer {
    /// Creates a new task buffer with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        TaskBuffer(Vec::with_capacity(capacity), HashMap::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
        self.1.clear();
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
        let v = utils::hash(&value.borrow());
        if let Some(ptr) = self.1.get(&v) {
            return *ptr;
        }

        let slice = self.extend_from_slice(value.borrow().as_bytes());
        let ptr = TaskBufferPtr {
            position: slice.position,
            size: slice.size,
            _phantom: PhantomData,
        };

        self.1.insert(v, ptr);
        ptr
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