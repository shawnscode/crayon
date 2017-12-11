use std::str;
use std::os::raw::c_void;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use gl;
use gl::types::*;

use utils::{Color, Rect};
use graphics::*;

use super::errors::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenGLBuffer {
    /// Vertex attributes.
    Vertex,
    /// Vertex array indices.
    Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VAOPair(GLuint, GLuint);

pub(crate) struct OpenGLVisitor {
    cull_face: Cell<CullFace>,
    front_face_order: Cell<FrontFaceOrder>,
    depth_test: Cell<Comparison>,
    depth_write: Cell<bool>,
    depth_write_offset: Cell<Option<(f32, f32)>>,
    color_blend: Cell<Option<(Equation, BlendFactor, BlendFactor)>>,
    color_write: Cell<(bool, bool, bool, bool)>,
    viewport: Cell<((u16, u16), (u16, u16))>,

    active_bufs: RefCell<HashMap<GLenum, GLuint>>,
    active_program: Cell<Option<GLuint>>,
    active_vao: Cell<Option<GLuint>>,
    active_textures: RefCell<[GLuint; MAX_UNIFORM_TEXTURE_SLOTS]>,
    active_framebuffer: Cell<GLuint>,
    active_renderbuffer: Cell<Option<GLuint>>,
    program_attribute_locations: RefCell<HashMap<GLuint, HashMap<String, GLint>>>,
    program_uniform_locations: RefCell<HashMap<GLuint, HashMap<String, GLint>>>,
    vertex_array_objects: RefCell<HashMap<VAOPair, GLuint>>,
}

impl OpenGLVisitor {
    pub unsafe fn new() -> OpenGLVisitor {
        // Reset all states to default.
        gl::Disable(gl::CULL_FACE);
        gl::FrontFace(gl::CCW);
        gl::Disable(gl::DEPTH_TEST);
        gl::DepthMask(gl::FALSE);
        gl::Disable(gl::POLYGON_OFFSET_FILL);
        gl::Disable(gl::BLEND);
        gl::ColorMask(1, 1, 1, 1);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

        OpenGLVisitor {
            cull_face: Cell::new(CullFace::Nothing),
            front_face_order: Cell::new(FrontFaceOrder::CounterClockwise),
            depth_test: Cell::new(Comparison::Always),
            depth_write: Cell::new(false),
            depth_write_offset: Cell::new(None),
            color_blend: Cell::new(None),
            color_write: Cell::new((false, false, false, false)),
            viewport: Cell::new(((0, 0), (128, 128))),

            active_bufs: RefCell::new(HashMap::new()),
            active_program: Cell::new(None),
            active_vao: Cell::new(None),
            active_textures: RefCell::new([0; MAX_UNIFORM_TEXTURE_SLOTS]),
            active_framebuffer: Cell::new(0), /* 0 makes sense here, for window's default frame buffer. */
            active_renderbuffer: Cell::new(None),
            program_attribute_locations: RefCell::new(HashMap::new()),
            program_uniform_locations: RefCell::new(HashMap::new()),
            vertex_array_objects: RefCell::new(HashMap::new()),
        }
    }

    pub unsafe fn bind_buffer(&self, tp: GLenum, id: GLuint) -> Result<()> {
        assert!(tp == gl::ARRAY_BUFFER || tp == gl::ELEMENT_ARRAY_BUFFER);

        // if let Some(record) = self.active_bufs.borrow().get(&tp) {
        //     if *record == id {
        //         return Ok(());
        //     }
        // }

        gl::BindBuffer(tp, id);
        self.active_bufs.borrow_mut().insert(tp, id);
        check()
    }

    pub unsafe fn bind_program(&self, id: GLuint) -> Result<()> {
        if let Some(record) = self.active_program.get() {
            if record == id {
                return Ok(());
            }
        }

        gl::UseProgram(id);
        self.active_program.set(Some(id));
        check()
    }

    pub unsafe fn bind_attribute_layout(&self,
                                        attributes: &AttributeLayout,
                                        layout: &VertexLayout)
                                        -> Result<()> {
        let pid = self.active_program.get().ok_or(ErrorKind::InvalidHandle)?;
        let vid = *self.active_bufs
                       .borrow()
                       .get(&gl::ARRAY_BUFFER)
                       .ok_or(ErrorKind::InvalidHandle)?;

        if let Some(vao) = self.vertex_array_objects.borrow().get(&VAOPair(pid, vid)) {
            if let Some(v) = self.active_vao.get() {
                if *vao == v {
                    return Ok(());
                }
            }

            gl::BindVertexArray(*vao);
            self.active_vao.set(Some(*vao));
            return check();
        }

        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        self.active_vao.set(Some(vao));

        for (name, size) in attributes.iter() {
            if let Some(element) = layout.element(name) {
                if element.size != size {
                    bail!(format!("vertex buffer has incompatible attribute {:?} [{:?} - {:?}].",
                                  name,
                                  element.size,
                                  size));
                }

                let offset = layout.offset(name).unwrap() as *const u8 as *const c_void;

                let location = self.get_uniform_location(pid, name.into())?;
                gl::EnableVertexAttribArray(location as GLuint);
                gl::VertexAttribPointer(location as GLuint,
                                        element.size as GLsizei,
                                        element.format.into(),
                                        element.normalized as u8,
                                        layout.stride() as GLsizei,
                                        offset);
            } else {
                bail!(format!("can't find attribute {:?} description in vertex buffer.",
                              name));
            }
        }

        check()?;
        self.vertex_array_objects
            .borrow_mut()
            .insert(VAOPair(pid, vid), vao);
        Ok(())
    }

    pub unsafe fn bind_uniform(&self, location: GLint, variable: &UniformVariable) -> Result<()> {
        match *variable {
            UniformVariable::Texture(_) => unreachable!(),
            UniformVariable::I32(v) => gl::Uniform1i(location, v),
            UniformVariable::F32(v) => gl::Uniform1f(location, v),
            UniformVariable::Vector2f(v) => gl::Uniform2f(location, v[0], v[1]),
            UniformVariable::Vector3f(v) => gl::Uniform3f(location, v[0], v[1], v[2]),
            UniformVariable::Vector4f(v) => gl::Uniform4f(location, v[0], v[1], v[2], v[3]),
            UniformVariable::Matrix2f(v, transpose) => {
                let transpose = if transpose { gl::TRUE } else { gl::FALSE };
                gl::UniformMatrix2fv(location, 1, transpose, v[0].as_ptr())
            }
            UniformVariable::Matrix3f(v, transpose) => {
                let transpose = if transpose { gl::TRUE } else { gl::FALSE };
                gl::UniformMatrix3fv(location, 1, transpose, v[0].as_ptr())
            }
            UniformVariable::Matrix4f(v, transpose) => {
                let transpose = if transpose { gl::TRUE } else { gl::FALSE };
                gl::UniformMatrix4fv(location, 1, transpose, v[0].as_ptr())
            }
        }

        check()
    }

    pub unsafe fn get_uniform_location(&self, id: GLuint, name: &str) -> Result<GLint> {
        let mut cache = self.program_uniform_locations.borrow_mut();
        if let Some(uniforms) = cache.get_mut(&id) {
            match uniforms.get(name).map(|v| *v) {
                Some(location) => Ok(location),
                None => {
                    let c_name = ::std::ffi::CString::new(name.as_bytes()).unwrap();
                    let location = gl::GetUniformLocation(id, c_name.as_ptr());
                    check()?;

                    uniforms.insert(name.to_string(), location);
                    Ok(location)
                }
            }
        } else {
            bail!(ErrorKind::InvalidHandle)
        }
    }

    pub unsafe fn get_attribute_location(&self, id: GLuint, name: &str) -> Result<GLint> {
        let mut cache = self.program_uniform_locations.borrow_mut();
        if let Some(attributes) = cache.get_mut(&id) {
            match attributes.get(name).map(|v| *v) {
                Some(location) => Ok(location),
                None => {
                    let c_name = ::std::ffi::CString::new(name.as_bytes()).unwrap();
                    let location = gl::GetAttribLocation(id, c_name.as_ptr());
                    check()?;

                    attributes.insert(name.to_string(), location);
                    Ok(location)
                }
            }
        } else {
            bail!(ErrorKind::InvalidHandle)
        }
    }

    pub unsafe fn clear(&self,
                        color: Option<Color>,
                        depth: Option<f32>,
                        stencil: Option<i32>)
                        -> Result<()> {

        let mut bits = 0;
        if let Some(v) = color {
            bits |= gl::COLOR_BUFFER_BIT;
            gl::ClearColor(v.0, v.1, v.2, v.3);
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

    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    pub unsafe fn set_viewport(&self, position: (u16, u16), size: (u16, u16)) -> Result<()> {
        if self.viewport.get().0 != position || self.viewport.get().1 != size {
            gl::Viewport(position.0 as i32,
                         position.1 as i32,
                         size.0 as i32,
                         size.1 as i32);
            self.viewport.set((position, size));
            check()
        } else {
            Ok(())
        }
    }

    /// Specify whether front- or back-facing polygons can be culled.
    pub unsafe fn set_cull_face(&self, face: CullFace) -> Result<()> {
        if self.cull_face.get() != face {
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

            self.cull_face.set(face);
            check()
        } else {
            Ok(())
        }
    }

    /// Define front- and back-facing polygons.
    pub unsafe fn set_front_face_order(&self, front: FrontFaceOrder) -> Result<()> {
        if self.front_face_order.get() != front {
            gl::FrontFace(match front {
                              FrontFaceOrder::Clockwise => gl::CW,
                              FrontFaceOrder::CounterClockwise => gl::CCW,
                          });
            self.front_face_order.set(front);
            check()
        } else {
            Ok(())
        }
    }

    /// Specify the value used for depth buffer comparisons.
    pub unsafe fn set_depth_test(&self, comparsion: Comparison) -> Result<()> {
        if self.depth_test.get() != comparsion {
            if comparsion != Comparison::Always {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(comparsion.into());
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }

            self.depth_test.set(comparsion);
            check()
        } else {
            Ok(())
        }
    }

    /// Enable or disable writing into the depth buffer.
    ///
    /// Optional `offset` to address the scale and units used to calculate depth values.
    pub unsafe fn set_depth_write(&self, enable: bool, offset: Option<(f32, f32)>) -> Result<()> {
        if self.depth_write.get() != enable {
            if enable {
                gl::DepthMask(gl::TRUE);
            } else {
                gl::DepthMask(gl::FALSE);
            }
            self.depth_write.set(enable);
        }

        if self.depth_write_offset.get() != offset {
            if let Some(v) = offset {
                if v.0 != 0.0 || v.1 != 0.0 {
                    gl::Enable(gl::POLYGON_OFFSET_FILL);
                    gl::PolygonOffset(v.0, v.1);
                } else {
                    gl::Disable(gl::POLYGON_OFFSET_FILL);
                }
            }
            self.depth_write_offset.set(offset);
        }

        check()
    }

    // Specifies how source and destination are combined.
    pub unsafe fn set_color_blend(&self,
                                  blend: Option<(Equation, BlendFactor, BlendFactor)>)
                                  -> Result<()> {

        if self.color_blend.get() != blend {
            if let Some((equation, src, dst)) = blend {
                if self.color_blend.get() == None {
                    gl::Enable(gl::BLEND);
                }

                gl::BlendFunc(src.into(), dst.into());
                gl::BlendEquation(equation.into());

            } else {
                if self.color_blend.get() != None {
                    gl::Disable(gl::BLEND);
                }
            }

            self.color_blend.set(blend);
            check()
        } else {
            Ok(())
        }
    }

    /// Enable or disable writing color elements into the color buffer.
    pub unsafe fn set_color_write(&self,
                                  red: bool,
                                  green: bool,
                                  blue: bool,
                                  alpha: bool)
                                  -> Result<()> {
        let cw = self.color_write.get();
        if cw.0 != red || cw.1 != green || cw.2 != blue || cw.3 != alpha {

            self.color_write.set((red, green, blue, alpha));
            gl::ColorMask(red as u8, green as u8, blue as u8, alpha as u8);
            check()
        } else {
            Ok(())
        }
    }

    pub unsafe fn create_program(&self, vs: &str, fs: &str) -> Result<GLuint> {
        let vs = self.compile(gl::VERTEX_SHADER, vs)?;
        let fs = self.compile(gl::FRAGMENT_SHADER, fs)?;
        let id = self.link(vs, fs)?;

        gl::DetachShader(id, vs);
        gl::DeleteShader(vs);
        gl::DetachShader(id, fs);
        gl::DeleteShader(fs);

        check()?;

        let mut cache = self.program_uniform_locations.borrow_mut();
        assert!(!cache.contains_key(&id));
        cache.insert(id, HashMap::new());

        let mut cache = self.program_attribute_locations.borrow_mut();
        assert!(!cache.contains_key(&id));
        cache.insert(id, HashMap::new());

        Ok(id)
    }

    pub unsafe fn delete_program(&self, id: GLuint) -> Result<()> {
        if let Some(v) = self.active_program.get() {
            if v == id {
                self.active_program.set(None);
            }
        }

        let vao_cache = &mut self.vertex_array_objects.borrow_mut();
        let mut removes = vec![];

        for pair in vao_cache.keys() {
            if pair.0 == id {
                removes.push(*pair);
            }
        }

        for pair in removes {
            if let Some(v) = vao_cache.remove(&pair) {
                gl::DeleteVertexArrays(1, &v);
                if let Some(vao) = self.active_vao.get() {
                    if v == vao {
                        self.active_vao.set(None);
                    }
                }
            }
        }

        gl::DeleteProgram(id);

        self.program_uniform_locations.borrow_mut().remove(&id);
        self.program_attribute_locations.borrow_mut().remove(&id);
        check()
    }

    pub unsafe fn bind_render_buffer(&self, id: GLuint) -> Result<()> {
        if id == 0 {
            bail!("failed to bind render buffer with 0.");
        }

        if let Some(v) = self.active_renderbuffer.get() {
            if v == id {
                return Ok(());
            }
        }

        gl::BindRenderbuffer(gl::RENDERBUFFER, id);
        self.active_renderbuffer.set(Some(id));
        check()
    }

    pub unsafe fn create_render_buffer(&self,
                                       format: GLenum,
                                       width: u32,
                                       height: u32)
                                       -> Result<GLuint> {
        let mut id = 0;
        gl::GenRenderbuffers(1, &mut id);
        assert!(id != 0);

        self.bind_render_buffer(id)?;
        gl::RenderbufferStorage(gl::RENDERBUFFER, format, width as GLint, height as GLint);
        check()?;
        Ok(id)
    }

    pub unsafe fn delete_render_buffer(&self, id: GLuint) -> Result<()> {
        gl::DeleteRenderbuffers(1, &id);
        check()
    }

    pub unsafe fn bind_texture(&self, slot: GLuint, id: GLuint) -> Result<()> {
        if id == 0 {
            bail!("failed to bind texture with 0.");
        }

        if slot as usize >= MAX_UNIFORM_TEXTURE_SLOTS {
            bail!("out of max texture slots.");
        }

        let cache = &mut self.active_textures.borrow_mut();
        if cache[slot as usize] != id {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, id);
            cache[slot as usize] = id;
            check()?;
        }

        Ok(())
    }

    pub unsafe fn create_texture(&self,
                                 internal_format: GLuint,
                                 format: GLenum,
                                 pixel_type: GLenum,
                                 address: TextureAddress,
                                 filter: TextureFilter,
                                 mipmap: bool,
                                 width: u32,
                                 height: u32,
                                 data: Option<&[u8]>)
                                 -> Result<(GLuint)> {
        let mut id = 0;
        gl::GenTextures(1, &mut id);
        assert!(id != 0);

        self.bind_texture(0, id)?;
        self.update_texture_parameters(address, filter, mipmap)?;

        let value = match data {
            Some(v) if v.len() > 0 => ::std::mem::transmute(&v[0]),
            _ => ::std::ptr::null(),
        };

        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       internal_format as GLint,
                       width as GLsizei,
                       height as GLsizei,
                       0,
                       format,
                       pixel_type,
                       value);

        if mipmap {
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        check()?;
        Ok(id)
    }

    pub unsafe fn update_texture(&self,
                                 id: GLuint,
                                 format: GLenum,
                                 tt: GLenum,
                                 rect: Rect,
                                 data: &[u8])
                                 -> Result<()> {
        self.bind_texture(0, id)?;

        gl::TexSubImage2D(gl::TEXTURE_2D,
                          0,
                          rect.min.x,
                          rect.min.y,
                          rect.width(),
                          rect.height(),
                          format,
                          tt,
                          ::std::mem::transmute(&data[0]));

        check()
    }

    pub unsafe fn update_texture_parameters(&self,
                                            address: TextureAddress,
                                            filter: TextureFilter,
                                            mipmap: bool)
                                            -> Result<()> {

        let address: GLenum = address.into();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, address as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, address as GLint);

        match filter {
            TextureFilter::Nearest => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            }
            TextureFilter::Linear => {
                if mipmap {
                    gl::TexParameteri(gl::TEXTURE_2D,
                                      gl::TEXTURE_MIN_FILTER,
                                      gl::LINEAR_MIPMAP_NEAREST as GLint);
                } else {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                }
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            }
        }

        check()
    }

    pub unsafe fn delete_texture(&self, id: GLuint) -> Result<()> {
        let cache = &mut self.active_textures.borrow_mut();
        for i in 0..MAX_UNIFORM_TEXTURE_SLOTS {
            if cache[i] == id {
                cache[i] = 0;
            }
        }

        gl::DeleteTextures(1, &id);
        check()
    }

    pub unsafe fn bind_framebuffer(&self, id: GLuint, check_status: bool) -> Result<()> {
        if self.active_framebuffer.get() == id {
            return Ok(());
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, id);

        if check_status && gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            self.active_framebuffer.set(0);
            bail!("framebuffer is not complete, fallback the to default framebuffer.");
        } else {
            self.active_framebuffer.set(id);
        }

        check()
    }

    pub unsafe fn bind_framebuffer_with_texture(&self, tp: GLenum, id: GLuint) -> Result<()> {
        if self.active_framebuffer.get() == 0 {
            bail!("cann't attach texture to default framebuffer.");
        }

        gl::FramebufferTexture2D(gl::FRAMEBUFFER, tp, gl::TEXTURE_2D, id, 0);
        check()
    }

    pub unsafe fn bind_framebuffer_with_renderbuffer(&self, tp: GLenum, id: GLuint) -> Result<()> {
        if self.active_framebuffer.get() == 0 {
            bail!("cann't attach render buffer to default framebuffer.");
        }

        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, tp, gl::RENDERBUFFER, id);
        check()
    }

    pub unsafe fn create_framebuffer(&self) -> Result<GLuint> {
        let mut id = 0;
        gl::GenFramebuffers(1, &mut id);
        assert!(id != 0);

        self.bind_framebuffer(id, false)?;
        check()?;
        Ok(id)
    }

    pub unsafe fn delete_framebuffer(&self, id: GLuint) -> Result<()> {
        if id == 0 {
            bail!("try to delete default frame buffer with id 0.");
        }

        if self.active_framebuffer.get() == id {
            self.bind_framebuffer(0, false)?;
        }

        gl::DeleteFramebuffers(1, &id);
        check()
    }

    pub unsafe fn create_buffer(&self,
                                buf: OpenGLBuffer,
                                hint: BufferHint,
                                size: u32,
                                data: Option<&[u8]>)
                                -> Result<GLuint> {
        let mut id = 0;
        gl::GenBuffers(1, &mut id);
        assert!(id != 0);

        self.bind_buffer(buf.into(), id)?;

        let value = match data {
            Some(v) if v.len() > 0 => ::std::mem::transmute(&v[0]),
            _ => ::std::ptr::null(),
        };

        gl::BufferData(buf.into(), size as isize, value, hint.into());
        check()?;
        Ok(id)
    }

    pub unsafe fn update_buffer(&self,
                                id: GLuint,
                                buf: OpenGLBuffer,
                                offset: u32,
                                data: &[u8])
                                -> Result<()> {
        self.bind_buffer(buf.into(), id)?;
        gl::BufferSubData(buf.into(),
                          offset as isize,
                          data.len() as isize,
                          ::std::mem::transmute(&data[0]));
        check()
    }

    pub unsafe fn delete_buffer(&self, id: GLuint) -> Result<()> {
        for (_, v) in self.active_bufs.borrow_mut().iter_mut() {
            if *v == id {
                *v = 0;
            }
        }

        let vao_cache = &mut self.vertex_array_objects.borrow_mut();
        let mut removes = vec![];

        for pair in vao_cache.keys() {
            if pair.1 == id {
                removes.push(*pair);
            }
        }

        for pair in removes {
            if let Some(v) = vao_cache.remove(&pair) {
                gl::DeleteVertexArrays(1, &v);
                if let Some(vao) = self.active_vao.get() {
                    if v == vao {
                        self.active_vao.set(None);
                    }
                }
            }
        }

        gl::DeleteBuffers(1, &id);
        check()
    }

    pub unsafe fn compile(&self, shader: GLenum, src: &str) -> Result<GLuint> {
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

    pub unsafe fn link(&self, vs: GLuint, fs: GLuint) -> Result<GLuint> {
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
}

pub unsafe fn check() -> Result<()> {
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

impl From<BufferHint> for GLenum {
    fn from(hint: BufferHint) -> Self {
        match hint {
            BufferHint::Immutable => gl::STATIC_DRAW,
            BufferHint::Stream => gl::STREAM_DRAW,
            BufferHint::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

impl From<OpenGLBuffer> for GLuint {
    fn from(res: OpenGLBuffer) -> Self {
        match res {
            OpenGLBuffer::Vertex => gl::ARRAY_BUFFER,
            OpenGLBuffer::Index => gl::ELEMENT_ARRAY_BUFFER,
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
            VertexFormat::Float => gl::FLOAT,
        }
    }
}

impl From<Primitive> for GLenum {
    fn from(primitive: Primitive) -> Self {
        match primitive {
            Primitive::Points => gl::POINTS,
            Primitive::Lines => gl::LINES,
            Primitive::LineStrip => gl::LINE_STRIP,
            Primitive::Triangles => gl::TRIANGLES,
            Primitive::TriangleStrip => gl::TRIANGLE_STRIP,
        }
    }
}

impl From<IndexFormat> for GLenum {
    fn from(format: IndexFormat) -> Self {
        match format {
            IndexFormat::U16 => gl::UNSIGNED_SHORT,
            IndexFormat::U32 => gl::UNSIGNED_INT,
        }
    }
}

impl From<TextureFormat> for (GLenum, GLenum, GLenum) {
    fn from(format: TextureFormat) -> Self {
        match format {
            TextureFormat::U8 => (gl::R8, gl::RED, gl::UNSIGNED_BYTE),
            TextureFormat::U8U8 => (gl::RG8, gl::RG, gl::UNSIGNED_BYTE),
            TextureFormat::U8U8U8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            TextureFormat::U8U8U8U8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            TextureFormat::U5U6U5 => (gl::RGB565, gl::RGB, gl::UNSIGNED_SHORT_5_6_5),
            TextureFormat::U4U4U4U4 => (gl::RGBA4, gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
            TextureFormat::U5U5U5U1 => (gl::RGB5_A1, gl::RGBA, gl::UNSIGNED_SHORT_5_5_5_1),
            TextureFormat::U10U10U10U2 => (gl::RGB10_A2, gl::RGBA, gl::UNSIGNED_INT_2_10_10_10_REV),
            TextureFormat::F16 => (gl::R16F, gl::RED, gl::HALF_FLOAT),
            TextureFormat::F16F16 => (gl::RG16F, gl::RG, gl::HALF_FLOAT),
            TextureFormat::F16F16F16 => (gl::RGB16F, gl::RGB, gl::HALF_FLOAT),
            TextureFormat::F16F16F16F16 => (gl::RGBA16F, gl::RGBA, gl::HALF_FLOAT),
            TextureFormat::F32 => (gl::R32F, gl::RED, gl::FLOAT),
            TextureFormat::F32F32 => (gl::RG32F, gl::RG, gl::FLOAT),
            TextureFormat::F32F32F32 => (gl::RGB32F, gl::RGB, gl::FLOAT),
            TextureFormat::F32F32F32F32 => (gl::RGBA32F, gl::RGBA, gl::FLOAT),
        }
    }
}

impl From<TextureAddress> for GLenum {
    fn from(address: TextureAddress) -> Self {
        match address {
            TextureAddress::Repeat => gl::REPEAT,
            TextureAddress::Mirror => gl::MIRRORED_REPEAT,
            TextureAddress::Clamp => gl::CLAMP_TO_EDGE,
            TextureAddress::MirrorClamp => gl::MIRROR_CLAMP_TO_EDGE,
        }
    }
}

impl From<RenderTextureFormat> for (GLenum, GLenum, GLenum) {
    fn from(format: RenderTextureFormat) -> Self {
        match format {
            RenderTextureFormat::RGB8 => (gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE),
            RenderTextureFormat::RGBA4 => (gl::RGBA4, gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
            RenderTextureFormat::RGBA8 => (gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE),
            RenderTextureFormat::Depth16 => {
                (gl::DEPTH_COMPONENT16, gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE)
            }
            RenderTextureFormat::Depth24 => {
                (gl::DEPTH_COMPONENT24, gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE)
            }
            RenderTextureFormat::Depth32 => {
                (gl::DEPTH_COMPONENT32, gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE)
            }
            RenderTextureFormat::Depth24Stencil8 => {
                (gl::DEPTH24_STENCIL8, gl::DEPTH_STENCIL, gl::UNSIGNED_BYTE)
            }

        }
    }
}