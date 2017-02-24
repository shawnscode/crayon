use std::sync::{Arc, RwLock, Mutex, MutexGuard};
use glutin;
use utility::HandleSet;

use super::*;
use super::errors::*;
use super::frame::*;
use super::resource::*;
use super::pipeline::*;
use super::color::Color;
use super::backend::Context;

pub struct Graphics {
    context: Context,

    views: HandleSet,
    pipelines: HandleSet,
    vertex_buffers: HandleSet,
    index_buffers: HandleSet,
    textures: HandleSet,

    frames: DoubleFrame,
    multithread: bool,
}

impl Graphics {
    pub fn new(window: Arc<glutin::Window>) -> Result<Self> {
        Ok(Graphics {
            context: Context::new(window)?,
            views: HandleSet::new(),
            pipelines: HandleSet::new(),
            vertex_buffers: HandleSet::new(),
            index_buffers: HandleSet::new(),
            textures: HandleSet::new(),
            frames: DoubleFrame::with_capacity(64 * 1024), // 64 kbs
            multithread: false,
        })
    }

    /// Advance to next frame. When using multithreaded renderer, this call just swaps internal
    /// buffers, kick render thread, and returns. In single threaded renderer this call does
    /// blocking frame rendering.
    pub fn run_one_frame(&mut self) -> Result<()> {
        unsafe {
            if !self.multithread {
                self.context.device().run_one_frame();
                self.frames.swap_frames();
                self.frames.back().dispatch(&mut self.context)?;
                self.frames.back().clear();
                self.context.swap_buffers()?;
            }

            Ok(())
        }
    }

    /// Creates an view with clear flags.
    ///
    /// View represent bucket of draw calls. Drawcalls inside bucket are sorted before
    /// submitting to underlaying OpenGL. In case where order has to be preserved (for
    /// example in rendering GUIs), view can be set to be in sequential order. Sequential
    /// order is less efficient, because it doesn't allow state change optimization, and
    /// should be avoided when possible.
    ///
    /// By default, views handles are ordered in ascending order. For dynamic renderers where
    /// order might not be known until the last moment, view handles can be remaped to arbitrary
    /// order by calling `update_view_order`.
    pub fn create_view(&mut self,
                       clear_color: Option<Color>,
                       clear_depth: Option<f32>,
                       clear_stencil: Option<i32>)
                       -> Result<ViewHandle> {
        let mut frame = self.frames.front();
        let handle = self.views.create().into();
        let ptr = frame.buf.extend(&ViewDesc {
            clear_color: clear_color.map(|v| v.into()),
            clear_depth: clear_depth,
            clear_stencil: clear_stencil,
        });

        frame.pre.push(PreFrameTask::CreateView(handle, ptr));
        Ok(handle)
    }

    /// TODO
    pub fn update_view_rect(&self) {}
    pub fn update_view_clear() {}
    pub fn update_view_sequential_mode() {}
    pub fn update_view_scissor() {}
    pub fn update_view_order() {}

    /// Destroy named view object.
    pub fn delete_view(&mut self, handle: ViewHandle) -> Result<()> {
        if self.views.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteView(handle));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Create a pipeline with initial shaders and render state. Pipeline encapusulate
    /// all the informations we need to configurate OpenGL before real drawing.
    pub fn create_pipeline(&mut self,
                           vs: &str,
                           fs: &str,
                           state: &RenderState,
                           attributes: &[VertexAttributeDesc])
                           -> Result<PipelineHandle> {
        let mut frame = self.frames.front();
        let handle = self.pipelines.create().into();

        let vs = frame.buf.extend_from_str(vs);
        let fs = frame.buf.extend_from_str(fs);
        let attributes = {
            let mut descs = [VertexAttributeDesc::default(); MAX_ATTRIBUTES];
            for (i, v) in attributes.iter().enumerate() {
                descs[i] = *v;
            }
            (attributes.len() as u8, descs)
        };

        let ptr = frame.buf.extend(&PipelineDesc {
            vs: vs,
            fs: fs,
            state: *state,
            attributes: attributes,
        });

        frame.pre.push(PreFrameTask::CreatePipeline(handle, ptr));
        Ok(handle)
    }

    /// Set the render state for all the drawcalls with this pipeline.
    pub fn update_pipeline_state(&mut self,
                                 handle: PipelineHandle,
                                 state: &RenderState)
                                 -> Result<()> {
        if self.pipelines.is_alive(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend(state);
            frame.pre.push(PreFrameTask::UpdatePipelineState(handle, ptr));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Destory internal pipeline state object.
    pub fn delete_pipeline(&mut self, handle: PipelineHandle) -> Result<()> {
        if self.pipelines.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeletePipeline(handle));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Create vertex buffer object with vertex layout declaration and optional data.
    pub fn create_vertex_buffer(&mut self,
                                layout: &VertexLayout,
                                hint: ResourceHint,
                                size: u32,
                                data: Option<&[u8]>)
                                -> Result<VertexBufferHandle> {
        let mut frame = self.frames.front();
        let handle = self.vertex_buffers.create().into();

        let data = data.map(|v| frame.buf.extend_from_slice(v));
        let ptr = frame.buf.extend(&VertexBufferDesc {
            layout: *layout,
            hint: hint,
            size: size,
            data: data,
        });

        frame.pre.push(PreFrameTask::CreateVertexBuffer(handle, ptr));
        Ok(handle)
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(&mut self,
                                handle: VertexBufferHandle,
                                offset: u32,
                                data: &[u8])
                                -> Result<()> {
        if self.vertex_buffers.is_alive(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            frame.pre.push(PreFrameTask::UpdateVertexBuffer(handle, offset, ptr));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Delete named vertex buffer.
    pub fn delete_vertex_buffer(&mut self, handle: VertexBufferHandle) -> Result<()> {
        if self.pipelines.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteVertexBuffer(handle));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Create index buffer object with optional data.
    pub fn create_index_buffer(&mut self,
                               format: IndexFormat,
                               hint: ResourceHint,
                               size: u32,
                               data: Option<&[u8]>)
                               -> Result<IndexBufferHandle> {
        let mut frame = self.frames.front();
        let handle = self.index_buffers.create().into();

        let data = data.map(|v| frame.buf.extend_from_slice(v));
        let ptr = frame.buf.extend(&IndexBufferDesc {
            format: format,
            hint: hint,
            size: size,
            data: data,
        });

        frame.pre.push(PreFrameTask::CreateIndexBuffer(handle, ptr));
        Ok(handle)
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_index_buffer(&mut self,
                               handle: IndexBufferHandle,
                               offset: u32,
                               data: &[u8])
                               -> Result<()> {
        if self.index_buffers.is_alive(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend_from_slice(data);
            frame.pre.push(PreFrameTask::UpdateIndexBuffer(handle, offset, ptr));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Delete named index buffer.
    pub fn delete_index_buffer(&mut self, handle: IndexBufferHandle) -> Result<()> {
        if self.index_buffers.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteIndexBuffer(handle));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// A texture is an image loaded in video memory, which can be sampled in shaders.
    pub fn create_texture(&mut self,
                          format: ColorTextureFormat,
                          address: TextureAddress,
                          filter: TextureFilter,
                          mipmap: bool,
                          width: u32,
                          height: u32,
                          data: &[u8])
                          -> Result<TextureHandle> {
        let mut frame = self.frames.front();
        let handle = self.textures.create().into();

        let data = frame.buf.extend_from_slice(data);
        let ptr = frame.buf.extend(&TextureDesc {
            format: format,
            address: address,
            filter: filter,
            mipmap: mipmap,
            width: width,
            height: height,
            data: data,
        });

        frame.pre.push(PreFrameTask::CreateTexture(handle, ptr));
        Ok(handle)
    }

    /// Update texture parameters.
    pub fn update_texture_parameters(&mut self,
                                     handle: TextureHandle,
                                     address: TextureAddress,
                                     filter: TextureFilter)
                                     -> Result<()> {
        if self.textures.is_alive(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.buf.extend(&TextureParametersDesc {
                address: address,
                filter: filter,
            });
            frame.pre.push(PreFrameTask::UpdateTextureParameters(handle, ptr));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Destroy named texture object.
    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<()> {
        if self.textures.free(handle) {
            let mut frame = self.frames.front();
            frame.post.push(PostFrameTask::DeleteTexture(handle));
            Ok(())
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }
}

impl Graphics {
    /// Submit primitive for drawing, within view all draw commands are executed after
    /// resource manipulation, such like `create_vertex_buffer`, `update_vertex_buffer`, etc.
    pub fn draw(&mut self,
                view: ViewHandle,
                pipeline: PipelineHandle,
                textures: &[(&str, TextureHandle)],
                uniforms: &[(&str, UniformVariable)],
                vb: VertexBufferHandle,
                ib: Option<IndexBufferHandle>,
                primitive: Primitive,
                from: u32,
                len: u32)
                -> Result<()> {
        let mut frame = self.frames.front();

        let uniforms = {
            let mut variables = vec![];
            for &(name, variable) in uniforms {
                variables.push((frame.buf.extend_from_str(name), variable));
            }
            frame.buf.extend_from_slice(variables.as_slice())
        };

        let textures = {
            let mut variables = vec![];
            for &(name, variable) in textures {
                variables.push((frame.buf.extend_from_str(name), variable));
            }
            frame.buf.extend_from_slice(variables.as_slice())
        };

        frame.drawcalls.push(FrameTask {
            view: view,
            pipeline: pipeline,
            primitive: primitive,
            vb: vb,
            ib: ib,
            from: from,
            len: len,
            textures: textures,
            uniforms: uniforms,
        });

        Ok(())
    }
}

struct DoubleFrame {
    idx: RwLock<usize>,
    frames: [Mutex<Frame>; 2],
}

impl DoubleFrame {
    pub fn with_capacity(capacity: usize) -> Self {
        DoubleFrame {
            idx: RwLock::new(0),
            frames: [Mutex::new(Frame::with_capacity(capacity)),
                     Mutex::new(Frame::with_capacity(capacity))],
        }
    }

    pub fn front(&self) -> MutexGuard<Frame> {
        self.frames[*self.idx.read().unwrap()].lock().unwrap()
    }

    pub fn back(&self) -> MutexGuard<Frame> {
        self.frames[*self.idx.read().unwrap()].lock().unwrap()
    }

    pub fn swap_frames(&self) {
        let mut idx = self.idx.write().unwrap();
        *idx = (*idx + 1) % 2;
    }
}