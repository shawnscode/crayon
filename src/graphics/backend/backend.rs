use std::str;
use std::borrow::Borrow;
use std::collections::HashMap;

use gl;
use gl::types::*;

use utility::Handle;

use super::*;
use super::super::pipeline::*;
use super::super::resource::*;

pub struct GLRenderState {
    viewport: ((u16, u16), (u16, u16)),
    cull_face: CullFace,
    front_face_order: FrontFaceOrder,
    depth_test: Comparison,
    depth_mask: bool,
    color_blend: Option<(Equation, BlendFactor, BlendFactor)>,
    color_mask: (bool, bool, bool, bool),
}

impl RenderStateVisitor for GLRenderState {
    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    unsafe fn set_viewport(&mut self, position: (u16, u16), size: (u16, u16)) -> Result<()> {
        if self.viewport.0 != position || self.viewport.1 != size {
            gl::Viewport(position.0 as i32,
                         position.1 as i32,
                         size.0 as i32,
                         size.1 as i32);
            self.viewport = (position, size);
        }

        check()
    }

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_face_cull(&mut self, face: CullFace) -> Result<()> {
        if self.cull_face != face {
            if face != CullFace::Nothing {
                gl::Enable(gl::CULL_FACE);
                gl::CullFace(match face {
                    CullFace::Front => gl::FRONT,
                    CullFace::Back => gl::BACK,
                    CullFace::Nothing => unreachable!(""),
                });
            } else {
                gl::Disable(gl::CULL_FACE);
            }

            self.cull_face = face;
        }

        check()
    }

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face(&mut self, front: FrontFaceOrder) -> Result<()> {
        if self.front_face_order != front {
            gl::FrontFace(match front {
                FrontFaceOrder::Clockwise => gl::CW,
                FrontFaceOrder::CounterClockwise => gl::CCW,
            });
            self.front_face_order = front;
        }

        check()
    }

    /// Specify the value used for depth buffer comparisons.
    unsafe fn set_depth_test(&mut self, comparsion: Comparison) -> Result<()> {
        if self.depth_test != comparsion {
            if comparsion != Comparison::Always {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(comparsion.into());
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }

            self.depth_test = comparsion;
        }

        check()
    }

    /// Enable or disable writing into the depth buffer.
    ///
    /// Optional `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write(&mut self, enable: bool, offset: Option<(f32, f32)>) -> Result<()> {
        if self.depth_mask != enable {
            if enable {
                gl::DepthMask(gl::TRUE);
            } else {
                gl::DepthMask(gl::FALSE);
            }
            self.depth_mask = enable;
        }

        if enable {
            if let Some(v) = offset {
                if v.0 != 0.0 || v.1 != 0.0 {
                    gl::Enable(gl::POLYGON_OFFSET_FILL);
                    gl::PolygonOffset(v.0, v.1);
                } else {
                    gl::Disable(gl::POLYGON_OFFSET_FILL);
                }
            }
        }

        check()
    }

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(&mut self,
                              blend: Option<(Equation, BlendFactor, BlendFactor)>)
                              -> Result<()> {
        if let Some((equation, src, dst)) = blend {
            if self.color_blend == None {
                gl::Enable(gl::BLEND);
            }

            if self.color_blend != blend {
                gl::BlendFunc(src.into(), dst.into());
                gl::BlendEquation(equation.into());
            }

        } else {
            if self.color_blend != None {
                gl::Disable(gl::BLEND);
            }
        }

        self.color_blend = blend;

        check()
    }

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(&mut self,
                              red: bool,
                              green: bool,
                              blue: bool,
                              alpha: bool)
                              -> Result<()> {
        if self.color_mask.0 != red || self.color_mask.1 != green || self.color_mask.2 != blue ||
           self.color_mask.3 != alpha {

            self.color_mask = (red, green, blue, alpha);
            gl::ColorMask(red as u8, green as u8, blue as u8, alpha as u8);
        }

        check()
    }
}

type ResourceID = GLuint;

#[derive(Debug, Clone, Copy)]
struct GLVertexBuffer {
    id: ResourceID,
    layout: VertexLayout,
    size: u32,
    hint: ResourceHint,
}

#[derive(Debug, Clone, Copy)]
struct GLIndexBuffer {
    id: ResourceID,
    format: IndexFormat,
    size: u32,
    hint: ResourceHint,
}

#[derive(Debug)]
struct GLProgram {
    id: ResourceID,
    attributes: Vec<(GLint, VertexAttributeDesc)>,
    uniforms: HashMap<String, GLint>,
}

pub struct GLBackend {
    vertex_buffers: DataVec<GLVertexBuffer>,
    index_buffers: DataVec<GLIndexBuffer>,
    pipelines: DataVec<GLProgram>,

    vbo: VertexBufferHandle,
    ibo: IndexBufferHandle,
    pso: PipelineHandle,
    vao_dirty: bool,
}

impl RasterizationStateVisitor for GLBackend {
    unsafe fn clear(&self,
                    color: Option<[f32; 4]>,
                    depth: Option<f32>,
                    stencil: Option<i32>)
                    -> Result<()> {
        let mut bits = 0;
        if let Some(v) = color {
            bits |= gl::COLOR_BUFFER_BIT;
            gl::ClearColor(v[0], v[1], v[2], v[3]);
        }

        if let Some(v) = depth {
            bits |= gl::DEPTH_BUFFER_BIT;
            gl::ClearDepth(v as f64);
        }

        if let Some(v) = stencil {
            bits |= gl::STENCIL_BUFFER_BIT;
            gl::ClearStencil(v);
        }

        gl::Clear(bits);
        check()
    }

    unsafe fn set_vertex_buffer(&mut self, handle: VertexBufferHandle) -> Result<()> {
        if let Some(vbo) = self.vertex_buffers.get(handle) {
            if handle != self.vbo {
                gl::BindBuffer(Resource::Vertex.into(), vbo.id);
                self.vbo = handle;
                self.vao_dirty = true;
                check()
            } else {
                Ok(())
            }
        } else {
            bail!(ErrorKind::InvalidHandle)
        }
    }

    unsafe fn set_index_buffer(&mut self, handle: IndexBufferHandle) -> Result<()> {
        if let Some(ibo) = self.index_buffers.get(handle) {
            if handle != self.ibo {
                gl::BindBuffer(Resource::Index.into(), ibo.id);
                self.ibo = handle;
                check()
            } else {
                Ok(())
            }
        } else {
            bail!(ErrorKind::InvalidHandle)
        }
    }

    unsafe fn set_program(&mut self, handle: PipelineHandle) -> Result<()> {
        if let Some(pso) = self.pipelines.get(handle) {
            if handle != self.pso {
                gl::UseProgram(pso.id);
                self.pso = handle;
                self.vao_dirty = true;
                check()
            } else {
                Ok(())
            }
        } else {
            bail!(ErrorKind::InvalidHandle)
        }
    }

    unsafe fn set_uniform(&mut self, name: &str, variable: &UniformVariable) -> Result<()> {
        let pso = self.pipelines
            .get_mut(self.pso)
            .ok_or(ErrorKind::Msg("pipeline object undefined.".to_string()))?;

        let location = match pso.uniforms.get(name).map(|v| *v) {
            Some(location) => location,
            None => {
                let c_name = ::std::ffi::CString::new(name.as_bytes()).unwrap();
                let location = gl::GetUniformLocation(pso.id, c_name.as_ptr());
                check()?;

                pso.uniforms.insert(name.to_string(), location);
                location
            }
        };

        if location != -1 {
            match *variable {
                UniformVariable::Vector1(v) => gl::Uniform1f(location, v[0]),
                UniformVariable::Vector2(v) => gl::Uniform2f(location, v[0], v[1]),
                UniformVariable::Vector3(v) => gl::Uniform3f(location, v[0], v[1], v[2]),
                UniformVariable::Vector4(v) => gl::Uniform4f(location, v[0], v[1], v[2], v[3]),
                _ => (),
            }
            check()
        } else {
            bail!("try to update undefined uniform {}.", name);
        }
    }

    unsafe fn commit(&mut self, primitive: Primitive, from: u32, len: u32) -> Result<()> {
        let pso = self.pipelines
            .get(self.pso)
            .ok_or(ErrorKind::Msg("pipeline object undefined.".to_string()))?;
        let vbo = self.vertex_buffers
            .get(self.vbo)
            .ok_or(ErrorKind::Msg("vertex buffer object undefined.".to_string()))?;

        if self.vao_dirty {
            for v in &pso.attributes {
                if let Some(element) = vbo.layout.element(v.1.name) {
                    if element.format != v.1.format || element.size != v.1.size {
                        bail!(format!("mismatch pipeline(format: {:?}({})) and vertex \
                                       buffer(format: {:?}({})).",
                                      v.1.format,
                                      v.1.size,
                                      element.format,
                                      element.size));
                    }

                    let offset = vbo.layout.offset(v.1.name).unwrap() as *const u8 as *const ::std::os::raw::c_void;
                    gl::EnableVertexAttribArray(v.0 as u32);
                    gl::VertexAttribPointer(v.0 as u32,
                                            element.size as i32,
                                            element.format.into(),
                                            element.normalized as u8,
                                            vbo.layout.stride() as i32,
                                            offset);
                } else {
                    bail!(format!("mismatch pipeline and vertex buffer. can't find attribute \
                                   named {:?}",
                                  v.1.name));
                }
            }
            check()?;
            self.vao_dirty = false;
        }

        if let Some(ibo) = self.index_buffers.get(self.ibo) {
            gl::DrawElements(primitive.into(),
                             len as i32,
                             ibo.format.into(),
                             from as *const u32 as *const ::std::os::raw::c_void);
        } else {
            gl::DrawArrays(primitive.into(), from as i32, len as i32);
        }
        check()
    }
}

impl ResourceStateVisitor for GLBackend {
    unsafe fn create_vertex_buffer(&mut self,
                                   handle: VertexBufferHandle,
                                   layout: &VertexLayout,
                                   hint: ResourceHint,
                                   size: u32,
                                   data: Option<&[u8]>)
                                   -> Result<()> {
        if self.vertex_buffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let vbo = GLVertexBuffer {
            id: create_buffer(Resource::Vertex, hint, size, data)?,
            layout: *layout,
            size: size,
            hint: hint,
        };

        self.vertex_buffers.set(handle, vbo);
        check()
    }

    unsafe fn update_vertex_buffer(&mut self,
                                   handle: VertexBufferHandle,
                                   offset: u32,
                                   data: &[u8])
                                   -> Result<()> {
        if let Some(vbo) = self.vertex_buffers.get(handle) {
            if vbo.hint == ResourceHint::Static {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() as u32 + offset > vbo.size {
                bail!(ErrorKind::OutOfBounds);
            }

            update_buffer(vbo.id, Resource::Vertex, offset, data)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    unsafe fn free_vertex_buffer(&mut self, handle: VertexBufferHandle) -> Result<()> {
        if let Some(vbo) = self.vertex_buffers.remove(handle) {
            free_buffer(vbo.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    unsafe fn create_index_buffer(&mut self,
                                  handle: IndexBufferHandle,
                                  format: IndexFormat,
                                  hint: ResourceHint,
                                  size: u32,
                                  data: Option<&[u8]>)
                                  -> Result<()> {
        if self.index_buffers.get(handle).is_some() {
            bail!(ErrorKind::DuplicatedHandle)
        }

        let ibo = GLIndexBuffer {
            id: create_buffer(Resource::Index, hint, size, data)?,
            format: format,
            size: size,
            hint: hint,
        };

        self.index_buffers.set(handle, ibo);
        check()
    }

    unsafe fn update_index_buffer(&self,
                                  handle: IndexBufferHandle,
                                  offset: u32,
                                  data: &[u8])
                                  -> Result<()> {
        if let Some(ibo) = self.index_buffers.get(handle) {
            if ibo.hint == ResourceHint::Static {
                bail!(ErrorKind::InvalidUpdateStaticResource);
            }

            if data.len() as u32 + offset > ibo.size {
                bail!(ErrorKind::OutOfBounds);
            }

            update_buffer(ibo.id, Resource::Index, offset, data)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    unsafe fn free_index_buffer(&mut self, handle: IndexBufferHandle) -> Result<()> {
        if let Some(ibo) = self.index_buffers.remove(handle) {
            free_buffer(ibo.id)
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }

    /// Initializes named program object. A program object is an object to
    /// which shader objects can be attached. Vertex and fragment shader
    /// are minimal requirement to build a proper program.
    unsafe fn create_pipeline(&mut self,
                              handle: PipelineHandle,
                              vs_src: &str,
                              fs_src: &str,
                              attributes: (u8, [VertexAttributeDesc; MAX_ATTRIBUTES]))
                              -> Result<()> {
        let vs = compile(gl::VERTEX_SHADER, vs_src)?;
        let fs = compile(gl::FRAGMENT_SHADER, fs_src)?;
        let id = link(vs, fs)?;

        gl::DetachShader(id, vs);
        gl::DeleteShader(vs);
        gl::DetachShader(id, fs);
        gl::DeleteShader(fs);

        let mut pipeline = GLProgram {
            id: id,
            attributes: Vec::new(),
            uniforms: HashMap::new(),
        };

        for i in 0..attributes.0 {
            let i = i as usize;
            let name: &'static str = attributes.1[i].name.into();
            let c_name = ::std::ffi::CString::new(name.as_bytes()).unwrap();
            let location = gl::GetAttribLocation(id, c_name.as_ptr());
            if location == -1 {
                bail!(format!("failed to GetAttribLocation for {}.", name));
            }

            pipeline.attributes.push((location, attributes.1[i]));
        }

        self.pipelines.set(handle, pipeline);
        check()
    }

    /// Free named program object.
    unsafe fn free_pipeline(&mut self, handle: PipelineHandle) -> Result<()> {
        if let Some(pso) = self.pipelines.remove(handle) {
            assert!(pso.id != 0);
            gl::DeleteProgram(pso.id);
            check()
        } else {
            bail!(ErrorKind::InvalidHandle);
        }
    }
}

unsafe fn create_buffer(buf: Resource,
                        hint: ResourceHint,
                        size: u32,
                        data: Option<&[u8]>)
                        -> Result<GLuint> {
    let mut id = 0;
    gl::GenBuffers(1, &mut id);
    if id == 0 {
        bail!("failed to create vertex buffer object.");
    }

    gl::BindBuffer(buf.into(), id);

    let value = match data {
        Some(v) if v.len() > 0 => ::std::mem::transmute(&v[0]),
        _ => ::std::ptr::null(),
    };

    gl::BufferData(buf.into(), size as isize, value, hint.into());
    check()?;
    Ok(id)
}

unsafe fn update_buffer(id: GLuint, buf: Resource, offset: u32, data: &[u8]) -> Result<()> {
    assert!(id != 0);
    gl::BindBuffer(buf.into(), id);
    gl::BufferSubData(buf.into(),
                      offset as isize,
                      data.len() as isize,
                      ::std::mem::transmute(&data[0]));
    check()
}

unsafe fn free_buffer(id: GLuint) -> Result<()> {
    assert!(id != 0);
    gl::DeleteBuffers(1, &id);
    check()
}

unsafe fn compile(shader: GLenum, src: &str) -> Result<GLuint> {
    let shader = gl::CreateShader(shader);
    // Attempt to compile the shader
    let c_str = ::std::ffi::CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ::std::ptr::null());
    gl::CompileShader(shader);

    // Get the compile status
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderInfoLog(shader,
                             len,
                             ::std::ptr::null_mut(),
                             buf.as_mut_ptr() as *mut GLchar);

        let error = format!("{}. with source:\n{}\n", str::from_utf8(&buf).unwrap(), src);
        bail!(ErrorKind::FailedCompilePipeline(error));
    }
    Ok(shader)
}

unsafe fn link(vs: GLuint, fs: GLuint) -> Result<GLuint> {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);

    gl::LinkProgram(program);
    // Get the link status
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(program,
                              len,
                              ::std::ptr::null_mut(),
                              buf.as_mut_ptr() as *mut GLchar);

        let error = format!("{}. ", str::from_utf8(&buf).unwrap());
        bail!(ErrorKind::FailedCompilePipeline(error));
    }
    Ok(program)
}

struct DataVec<T>
    where T: Sized
{
    buf: Vec<Option<T>>,
}

impl<T> DataVec<T>
    where T: Sized
{
    pub fn get<H>(&self, handle: H) -> Option<&T>
        where H: Borrow<Handle>
    {
        self.buf.get(handle.borrow().index() as usize).and_then(|v| v.as_ref())
    }

    pub fn get_mut<H>(&mut self, handle: H) -> Option<&mut T>
        where H: Borrow<Handle>
    {
        self.buf.get_mut(handle.borrow().index() as usize).and_then(|v| v.as_mut())
    }

    pub fn set<H>(&mut self, handle: H, value: T)
        where H: Borrow<Handle>
    {
        let handle = handle.borrow();
        while self.buf.len() <= handle.index() as usize {
            self.buf.push(None);
        }

        self.buf[handle.index() as usize] = Some(value);
    }

    pub fn remove<H>(&mut self, handle: H) -> Option<T>
        where H: Borrow<Handle>
    {
        let handle = handle.borrow();
        if self.buf.len() <= handle.index() as usize {
            None
        } else {
            let mut value = None;
            ::std::mem::swap(&mut value, &mut self.buf[handle.index() as usize]);
            value
        }
    }
}

unsafe fn check() -> Result<()> {
    match gl::GetError() {
        gl::NO_ERROR => Ok(()),
        gl::INVALID_ENUM => Err(ErrorKind::InvalidEnum.into()),
        gl::INVALID_VALUE => Err(ErrorKind::InvalidValue.into()),
        gl::INVALID_OPERATION => Err(ErrorKind::InvalidOperation.into()),
        gl::INVALID_FRAMEBUFFER_OPERATION => Err(ErrorKind::InvalidFramebufferOperation.into()),
        gl::OUT_OF_MEMORY => Err(ErrorKind::OutOfBounds.into()),
        _ => Err(ErrorKind::Unknown.into()),
    }
}

impl From<ResourceHint> for GLenum {
    fn from(hint: ResourceHint) -> Self {
        match hint {
            ResourceHint::Static => gl::STATIC_DRAW,
            ResourceHint::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

impl From<Resource> for GLuint {
    fn from(res: Resource) -> Self {
        match res {
            Resource::Vertex => gl::ARRAY_BUFFER,
            Resource::Index => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}


impl From<Comparison> for GLenum {
    fn from(cmp: Comparison) -> Self {
        match cmp {
            Comparison::Never => gl::NEVER,
            Comparison::Less => gl::LESS,
            Comparison::LessOrEqual => gl::LEQUAL,
            Comparison::Greater => gl::GREATER,
            Comparison::GreaterOrEqual => gl::GEQUAL,
            Comparison::Equal => gl::EQUAL,
            Comparison::NotEqual => gl::NOTEQUAL,
            Comparison::Always => gl::ALWAYS,
        }
    }
}

impl From<Equation> for GLenum {
    fn from(eq: Equation) -> Self {
        match eq {
            Equation::Add => gl::FUNC_ADD,
            Equation::Subtract => gl::FUNC_SUBTRACT,
            Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for GLenum {
    fn from(factor: BlendFactor) -> Self {
        match factor {
            BlendFactor::Zero => gl::ZERO,
            BlendFactor::One => gl::ONE,
            BlendFactor::Value(BlendValue::SourceColor) => gl::SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => gl::SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => gl::DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => gl::DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => gl::ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => gl::ONE_MINUS_DST_ALPHA,
        }
    }
}

impl From<VertexFormat> for GLenum {
    fn from(format: VertexFormat) -> Self {
        match format {
            VertexFormat::Byte => gl::BYTE,
            VertexFormat::UByte => gl::UNSIGNED_BYTE,
            VertexFormat::Short => gl::SHORT,
            VertexFormat::UShort => gl::UNSIGNED_SHORT,
            VertexFormat::Fixed => gl::FIXED,
            VertexFormat::Float => gl::FLOAT,
        }
    }
}

impl From<Primitive> for GLenum {
    fn from(primitive: Primitive) -> Self {
        match primitive {
            Primitive::Points => gl::POINTS,
            Primitive::Lines => gl::LINES,
            Primitive::LineLoop => gl::LINE_LOOP,
            Primitive::LineStrip => gl::LINE_STRIP,
            Primitive::Triangles => gl::TRIANGLES,
            Primitive::TriangleFan => gl::TRIANGLE_FAN,
            Primitive::TriangleStrip => gl::TRIANGLE_STRIP,
        }
    }
}

impl From<IndexFormat> for GLenum {
    fn from(format: IndexFormat) -> Self {
        match format {
            IndexFormat::UByte => gl::UNSIGNED_BYTE,
            IndexFormat::UShort => gl::UNSIGNED_SHORT,
        }
    }
}