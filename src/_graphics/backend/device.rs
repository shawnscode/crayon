use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;

use gl;
use gl::types::*;
use utility::{Handle, HandleObjectSet};
use super::*;

pub struct Device {
    viewport: ((u32, u32), (u32, u32)),
    cull_face: CullFace,
    front_face_order: FrontFaceOrder,
    depth_test: Comparison,
    depth_mask: bool,
    color_blend: Option<(Equation, BlendFactor, BlendFactor)>,
    color_mask: (bool, bool, bool, bool),

    buffers: HandleObjectSet<BufferGL>,
    programs: HandleObjectSet<ProgramGL>,
}

impl Device {
    pub fn new() -> Self {
        Device {
            viewport: ((0, 0), (0, 0)),
            cull_face: CullFace::Nothing,
            front_face_order: FrontFaceOrder::CounterClockwise,
            depth_test: Comparison::Always,
            depth_mask: false,
            color_blend: None,
            color_mask: (true, true, true, true),
            buffers: HandleObjectSet::new(),
            programs: HandleObjectSet::new(),
        }
    }

    pub fn run_one_frame(&mut self) {}

    pub unsafe fn check() -> Result<(), Error> {
        match gl::GetError() {
            gl::NO_ERROR => Ok(()),
            other => Err(Error::from(other)),
        }
    }
}

impl RenderStateVisitor for Device {
    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    ///
    /// When a GL context is first attached to a window, size is set to the
    /// dimensions of that window and initial position is (0, 0).
    unsafe fn set_viewport(&mut self, position: (u32, u32), size: (u32, u32)) {
        if self.viewport.0 != position || self.viewport.1 != size {
            gl::Viewport(position.0 as i32,
                         position.1 as i32,
                         size.0 as i32,
                         size.1 as i32);
            self.viewport = (position, size);
        }

        Device::check().unwrap();
    }

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_face_cull(&mut self, face: CullFace) {
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

        Device::check().unwrap();
    }

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face(&mut self, front: FrontFaceOrder) {
        if self.front_face_order != front {
            gl::FrontFace(match front {
                FrontFaceOrder::Clockwise => gl::CW,
                FrontFaceOrder::CounterClockwise => gl::CCW,
            });
            self.front_face_order = front;
        }

        Device::check().unwrap();
    }

    /// Specify the value used for depth buffer comparisons.
    unsafe fn set_depth_test(&mut self, comparsion: Comparison) {
        if self.depth_test != comparsion {
            if comparsion != Comparison::Always {
                gl::Enable(gl::DEPTH_TEST);
                gl::DepthFunc(comparsion.to_native());
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }

            self.depth_test = comparsion;
        }

        Device::check().unwrap();
    }

    /// Enable or disable writing into the depth buffer.
    ///
    /// Optional `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write(&mut self, enable: bool, offset: Option<(f32, f32)>) {
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

        Device::check().unwrap();
    }

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(&mut self, blend: Option<(Equation, BlendFactor, BlendFactor)>) {
        if let Some((equation, src, dst)) = blend {
            if self.color_blend == None {
                gl::Enable(gl::BLEND);
            }

            if self.color_blend != blend {
                gl::BlendFunc(src.to_native(), dst.to_native());
                gl::BlendEquation(equation.to_native());
            }

        } else {
            if self.color_blend != None {
                gl::Disable(gl::BLEND);
            }
        }

        self.color_blend = blend;
        Device::check().unwrap();
    }

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(&mut self, red: bool, green: bool, blue: bool, alpha: bool) {
        if self.color_mask.0 != red || self.color_mask.1 != green || self.color_mask.2 != blue ||
           self.color_mask.3 != alpha {

            self.color_mask = (red, green, blue, alpha);
            gl::ColorMask(red as u8, green as u8, blue as u8, alpha as u8);
        }

        Device::check().unwrap();
    }
}

type ResourceID = GLuint;

#[derive(Debug, Clone, Copy)]
struct BufferGL {
    id: ResourceID,
    size: u32,
    buffer: Buffer,
    hint: BufferHint,
}

impl Default for BufferGL {
    fn default() -> Self {
        BufferGL {
            id: 0,
            size: 0,
            buffer: Buffer::Vertex,
            hint: BufferHint::Static,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ProgramGL {
    id: ResourceID, /* attributes: HashMap<String, GLsizei>,
                     * uniforms: HashMap<String, GLsizei>,
                     * textures: HashMap<String, GLsizei>, */
}

impl ResourceStateVisitor for Device {
    /// Initialize buffer, named by `handle`, with optional initial data.
    unsafe fn create_buffer(&mut self,
                            buffer: Buffer,
                            hint: BufferHint,
                            size: u32,
                            data: Option<&[u8]>)
                            -> Handle {
        let mut id = 0;
        gl::GenBuffers(1, &mut id);
        assert!(id != 0, "Failed to generate buffer object.");

        gl::BindBuffer(buffer.to_native(), id);

        let value = match data {
            Some(v) if v.len() > 0 => mem::transmute(&v[0]),
            _ => ptr::null(),
        };

        gl::BufferData(buffer.to_native(), size as isize, value, hint.to_native());
        Device::check().unwrap();

        self.buffers.create(BufferGL {
            id: id,
            buffer: buffer,
            hint: hint,
            size: size,
        })
    }

    /// Update named dynamic `MemoryHint::Dynamic` buffer.
    ///
    /// Optional `offset` to specifies the offset into the buffer object's data
    /// store where data replacement will begin, measured in bytes.
    unsafe fn update_buffer(&mut self, handle: Handle, offset: u32, data: &[u8]) {
        if let Some(bo) = self.buffers.get(handle) {
            if bo.id != 0 {
                assert!(bo.hint == BufferHint::Dynamic,
                        "Try to update static buffer.");
                assert!(data.len() as u32 + offset >= bo.size, "Out of bounds.");

                gl::BindBuffer(bo.buffer.to_native(), bo.id);
                gl::BufferSubData(bo.buffer.to_native(),
                                  offset as isize,
                                  data.len() as isize,
                                  mem::transmute(&data[0]));

                Device::check().unwrap();
            }
        }
    }

    /// Free named buffer object.
    unsafe fn free_buffer(&mut self, handle: Handle) {

        if let Some(bo) = self.buffers.get(handle) {
            if bo.id != 0 {
                gl::DeleteBuffers(1, &bo.id);
                Device::check().unwrap();
            }
        }

        self.buffers.free(handle);
    }

    /// Initializes named program object. A program object is an object to
    /// which shader objects can be attached. Vertex and fragment shader
    /// are minimal requirement to build a proper program.
    unsafe fn create_program(&mut self,
                             vs_src: &str,
                             fs_src: &str,
                             gs_src: Option<&str>)
                             -> Handle {
        let vs = compile(gl::VERTEX_SHADER, vs_src);
        let fs = compile(gl::FRAGMENT_SHADER, fs_src);
        let gs = if let Some(v) = gs_src {
            Some(compile(gl::GEOMETRY_SHADER, v))
        } else {
            None
        };

        let id = link(vs, fs, gs);
        assert!(id != 0, "Failed to generate program object.");

        gl::DetachShader(id, vs);
        gl::DeleteShader(vs);
        gl::DetachShader(id, fs);
        gl::DeleteShader(fs);

        if let Some(v) = gs {
            gl::DetachShader(id, v);
            gl::DeleteShader(v);
        }

        Device::check().unwrap();

        self.programs.create(ProgramGL { id: id })
    }

    /// Free named program object.
    unsafe fn free_program(&mut self, handle: Handle) {
        if let Some(po) = self.programs.get(handle) {
            if po.id != 0 {
                gl::DeleteProgram(po.id);
                Device::check().unwrap();
            }
        }

        self.programs.free(handle);
    }
}

unsafe fn compile(shader: GLenum, src: &str) -> GLuint {
    let shader = gl::CreateShader(shader);
    // Attempt to compile the shader
    let c_str = CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
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
                             ptr::null_mut(),
                             buf.as_mut_ptr() as *mut GLchar);
        panic!("Failed to compile, {}. source:\n{}\n",
               str::from_utf8(&buf).unwrap(),
               src);
    }
    shader
}

unsafe fn link(vs: GLuint, fs: GLuint, gs: Option<GLuint>) -> GLuint {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);

    if let Some(v) = gs {
        gl::AttachShader(program, v);
    }

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
                              ptr::null_mut(),
                              buf.as_mut_ptr() as *mut GLchar);
        panic!("Failed to link program, {}.", str::from_utf8(&buf).unwrap());
    }
    program
}

impl RasterizationStateVisitor for Device {
    /// Clear any or all of rendertarget, depth buffer and stencil buffer.
    unsafe fn clear(&self, color: Option<[f32; 4]>, depth: Option<f32>, stencil: Option<i32>) {
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
    }

    /// Bind a named buffer object.
    unsafe fn bind_buffer(&mut self, handle: Handle) {
        if let Some(bo) = self.buffers.get(handle) {
            gl::BindBuffer(bo.buffer.to_native(), bo.id);
        }
    }

    /// Bind a named program object.
    unsafe fn bind_program(&mut self, handle: Handle) {
        if let Some(po) = self.programs.get(handle) {
            gl::UseProgram(po.id);
        }
    }

    /// Bind a named texture object into sampler unit.
    unsafe fn bind_texture(&mut self, _: Handle, _: u32) {}

    /// Commit render primitives from binding data.
    unsafe fn commit(_: Primitive, _: u32, _: u32) {}
}