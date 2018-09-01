use gl;
use gl::types::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use application::window::Window;
use errors::*;
use math;
use utils::hash_value;

use super::super::super::assets::prelude::*;
use super::super::super::MAX_UNIFORM_TEXTURE_SLOTS;
use super::super::{UniformVar, Visitor};
use super::capabilities::{Capabilities, Version};
use super::types::DataVec;

#[derive(Debug, Clone)]
struct GLSurfaceFBO {
    id: GLuint,
    dimensions: math::Vector2<u32>,
}

#[derive(Debug, Clone)]
struct GLSurface {
    fbo: Option<GLSurfaceFBO>,
    params: SurfaceParams,
}

#[derive(Debug, Clone)]
struct GLShader {
    id: GLuint,
    params: ShaderParams,
    uniforms: RefCell<HashMap<hash_value::HashValue<str>, GLint>>,
    attributes: RefCell<HashMap<hash_value::HashValue<str>, GLint>>,
}

impl GLShader {
    fn hash_uniform_location<T: Into<hash_value::HashValue<str>>>(&self, name: T) -> Option<GLint> {
        self.uniforms.borrow().get(&name.into()).cloned()
    }

    unsafe fn uniform_location(&self, name: &str) -> Result<GLint> {
        let hash = name.into();
        let mut uniforms = self.uniforms.borrow_mut();
        match uniforms.get(&hash).cloned() {
            Some(location) => Ok(location),
            None => {
                let c_name = ::std::ffi::CString::new(name.as_bytes()).unwrap();
                let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
                check()?;

                uniforms.insert(hash, location);
                Ok(location)
            }
        }
    }

    unsafe fn attribute_location(&self, name: &str) -> Result<GLint> {
        let hash = name.into();
        let mut attributes = self.attributes.borrow_mut();
        match attributes.get(&hash).cloned() {
            Some(location) => Ok(location),
            None => {
                let c_name = ::std::ffi::CString::new(name.as_bytes()).unwrap();
                let location = gl::GetAttribLocation(self.id, c_name.as_ptr());
                check()?;

                attributes.insert(hash, location);
                Ok(location)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct GLMesh {
    vbo: GLuint,
    ibo: GLuint,
    params: MeshParams,
}

#[derive(Debug, Copy, Clone)]
struct GLTexture {
    id: GLuint,
    params: TextureParams,
    allocated: bool,
}

#[derive(Debug, Copy, Clone)]
struct GLRenderTexture {
    id: GLuint,
    params: RenderTextureParams,
}

struct GLVisitorMutInternal {
    render_state: RenderState,
    scissor: SurfaceScissor,
    view: SurfaceViewport,
    binded_render_buffer: Option<GLuint>,
    binded_buffers: HashMap<GLenum, GLuint>,
    binded_vao: Option<GLuint>,
    binded_surface: Option<SurfaceHandle>,
    binded_framebuffer: Option<GLuint>,
    binded_frame_surfaces: HashSet<SurfaceHandle>,
    binded_shader: Option<GLuint>,
    binded_texture_index: usize,
    binded_textures: [Option<GLuint>; MAX_UNIFORM_TEXTURE_SLOTS],
    vaos: HashMap<(GLuint, GLuint), GLuint>,
}

pub struct GLVisitor {
    mutables: RefCell<GLVisitorMutInternal>,
    surfaces: DataVec<GLSurface>,
    shaders: DataVec<GLShader>,
    meshes: DataVec<GLMesh>,
    textures: DataVec<GLTexture>,
    render_textures: DataVec<GLRenderTexture>,
    capabilities: Capabilities,
}

impl GLVisitor {
    pub unsafe fn new(window: &Window) -> Result<Self> {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        let capabilities = Capabilities::parse()?;
        info!("GLVisitor {:#?}", capabilities);
        check_capabilities(&capabilities)?;

        let mutables = GLVisitorMutInternal {
            render_state: RenderState::default(),
            scissor: SurfaceScissor::Disable,
            view: SurfaceViewport {
                position: math::Vector2::new(0, 0),
                size: math::Vector2::new(0, 0),
            },
            binded_render_buffer: None,
            binded_buffers: HashMap::new(),
            binded_vao: None,
            binded_surface: None,
            binded_framebuffer: None,
            binded_frame_surfaces: HashSet::new(),
            binded_shader: None,
            binded_texture_index: 0,
            binded_textures: [None; MAX_UNIFORM_TEXTURE_SLOTS],
            vaos: HashMap::new(),
        };

        let visitor = GLVisitor {
            mutables: RefCell::new(mutables),
            surfaces: DataVec::new(),
            shaders: DataVec::new(),
            meshes: DataVec::new(),
            textures: DataVec::new(),
            render_textures: DataVec::new(),
            capabilities: capabilities,
        };

        visitor.reset_render_state()?;
        Ok(visitor)
    }
}

impl Visitor for GLVisitor {
    unsafe fn advance(&mut self) -> Result<()> {
        {
            let mut mutables = self.mutables.borrow_mut();
            mutables.binded_frame_surfaces.clear();
            mutables.binded_surface = None;
        }

        Ok(())
    }

    unsafe fn create_surface(
        &mut self,
        handle: SurfaceHandle,
        params: SurfaceParams,
    ) -> Result<()> {
        let fbo = if params.colors[0].is_some() || params.depth_stencil.is_some() {
            let mut id = 0;
            gl::GenFramebuffers(1, &mut id);
            assert!(id != 0);

            self.bind_framebuffer(id, false)?;

            let mut dimensions = None;
            for (i, attachment) in params.colors.iter().enumerate() {
                if let Some(v) = *attachment {
                    let rt = self.render_textures
                        .get(v)
                        .ok_or_else(|| format_err!("RenderTexture handle {:?} is invalid.", v))?;

                    if !rt.params.format.is_color() {
                        bail!(
                            "Incompitable(mismatch format) attachments of SurfaceObject {:?}",
                            id
                        );
                    }

                    if dimensions.is_some() && dimensions != Some(rt.params.dimensions) {
                        bail!(
                            "Incompitable(mismatch dimensons) attachments of SurfaceObject {:?}",
                            id
                        );
                    }

                    dimensions = Some(rt.params.dimensions);
                    self.update_framebuffer_render_texture(rt.id, rt.params, i)?;
                }
            }

            if let Some(v) = params.depth_stencil {
                let rt = self.render_textures
                    .get(v)
                    .ok_or_else(|| format_err!("RenderTexture handle {:?} is invalid.", v))?;

                if rt.params.format.is_color() {
                    bail!(
                        "Incompitable(mismatch format) attachments of SurfaceObject {:?}",
                        id
                    );
                }

                if dimensions.is_some() && dimensions != Some(rt.params.dimensions) {
                    bail!(
                        "Incompitable(mismatch dimensions) attachments of SurfaceObject {:?}",
                        id
                    );
                }

                dimensions = Some(rt.params.dimensions);
                self.update_framebuffer_render_texture(rt.id, rt.params, 0)?;
            }

            Some(GLSurfaceFBO {
                id: id,
                dimensions: dimensions.unwrap(),
            })
        } else {
            None
        };

        self.surfaces.create(
            handle,
            GLSurface {
                fbo: fbo,
                params: params,
            },
        );

        Ok(())
    }

    unsafe fn delete_surface(&mut self, handle: SurfaceHandle) -> Result<()> {
        let surface = self.surfaces
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if let Some(fbo) = surface.fbo {
            assert!(fbo.id != 0);

            if self.mutables.borrow().binded_framebuffer == Some(fbo.id) {
                self.bind_framebuffer(0, false)?;
            }

            gl::DeleteFramebuffers(1, &fbo.id);
            check()?;
        }

        Ok(())
    }

    unsafe fn create_shader(
        &mut self,
        handle: ShaderHandle,
        params: ShaderParams,
        vs: &str,
        fs: &str,
    ) -> Result<()> {
        let vs = self.compile(gl::VERTEX_SHADER, vs)?;
        let fs = self.compile(gl::FRAGMENT_SHADER, fs)?;
        let id = self.link(vs, fs)?;

        gl::DetachShader(id, vs);
        gl::DeleteShader(vs);
        gl::DetachShader(id, fs);
        gl::DeleteShader(fs);
        check()?;

        let shader = GLShader {
            id: id,
            params: params,
            uniforms: RefCell::new(HashMap::new()),
            attributes: RefCell::new(HashMap::new()),
        };

        for (name, _, _) in shader.params.attributes.iter() {
            let name: &'static str = name.into();
            let location = shader.attribute_location(name)?;
            if location == -1 {
                self.delete_shader_intern(id)?;
                bail!("Attribute({:?}) is undefined in shader sources.", name);
            }
        }

        for &(ref name, _) in shader.params.uniforms.iter() {
            let location = shader.uniform_location(name)?;
            if location == -1 {
                self.delete_shader_intern(id)?;
                bail!("Uniform({:?}) is undefined in shader sources.", name);
            }
        }

        self.shaders.create(handle, shader);
        Ok(())
    }

    unsafe fn delete_shader(&mut self, handle: ShaderHandle) -> Result<()> {
        let shader = self.shaders
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        // Removes deprecated `VertexArrayObject`s.
        self.mutables
            .borrow_mut()
            .vaos
            .retain(|&(sid, _), _| sid != shader.id);

        self.delete_shader_intern(shader.id)
    }

    unsafe fn create_texture(
        &mut self,
        handle: TextureHandle,
        params: TextureParams,
        data: Option<TextureData>,
    ) -> Result<()> {
        // Maybe we should implements some software decoder for common texture compression format.
        if !params.format.is_support(&self.capabilities) {
            bail!(
                "The GL Context does not support the texture format {:?}.",
                params.format
            );
        }

        let mut id = 0;
        gl::GenTextures(1, &mut id);
        assert!(id != 0);

        let (internal_format, format, pixel_type) = params.format.into();
        let is_compression = params.format.is_compression();
        let mut allocated = false;

        if let Some(mut data) = data {
            let len = data.bytes.len();
            if len > 0 {
                self.bind_texture(0, id)?;
                self.update_texture_params(id, params.wrap, params.filter, len as u32)?;

                let mut dims = (
                    params.dimensions.x as GLsizei,
                    params.dimensions.y as GLsizei,
                );

                if is_compression {
                    for (i, v) in data.bytes.drain(..).enumerate() {
                        gl::CompressedTexImage2D(
                            gl::TEXTURE_2D,
                            i as GLint,
                            internal_format,
                            dims.0,
                            dims.1,
                            0,
                            v.len() as GLint,
                            &v[0] as *const u8 as *const ::std::os::raw::c_void,
                        );

                        dims.0 = (dims.0 / 2).max(1);
                        dims.1 = (dims.1 / 2).max(1);
                    }
                } else {
                    for (i, v) in data.bytes.drain(..).enumerate() {
                        gl::TexImage2D(
                            gl::TEXTURE_2D,
                            i as GLint,
                            internal_format as GLint,
                            dims.0,
                            dims.1,
                            0,
                            format,
                            pixel_type,
                            &v[0] as *const u8 as *const ::std::os::raw::c_void,
                        );

                        dims.0 = (dims.0 / 2).max(1);
                        dims.1 = (dims.1 / 2).max(1);
                    }
                }

                allocated = true;
            }
        }

        check()?;

        self.textures.create(
            handle,
            GLTexture {
                id: id,
                params: params,
                allocated: allocated,
            },
        );

        Ok(())
    }

    unsafe fn update_texture(
        &mut self,
        handle: TextureHandle,
        area: math::Aabb2<u32>,
        data: &[u8],
    ) -> Result<()> {
        let texture = *self.textures
            .get(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if texture.params.hint == TextureHint::Immutable {
            bail!("Trying to update immutable texture.");
        }

        if texture.params.format.is_compression() {
            bail!("Trying to update compressed texture.");
        }

        if data.len() > area.volume() as usize
            || area.min.x >= texture.params.dimensions.x
            || area.min.y >= texture.params.dimensions.y
        {
            bail!("Trying to update texture data out of bounds.");
        }

        let (internal_format, format, pixel_type) = texture.params.format.into();

        self.bind_texture(0, texture.id)?;

        if !texture.allocated {
            self.update_texture_params(texture.id, texture.params.wrap, texture.params.filter, 1)?;

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as GLint,
                texture.params.dimensions.x as GLsizei,
                texture.params.dimensions.y as GLsizei,
                0,
                format,
                pixel_type,
                ::std::ptr::null(),
            );

            self.textures.get_mut(handle).unwrap().allocated = true;
        }

        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            area.min.x as i32,
            area.min.y as i32,
            area.dim().x as i32,
            area.dim().y as i32,
            format,
            pixel_type,
            &data[0] as *const u8 as *const ::std::os::raw::c_void,
        );

        check()
    }

    unsafe fn delete_texture(&mut self, handle: TextureHandle) -> Result<()> {
        let texture = self.textures
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;
        self.delete_texture_intern(texture.id)
    }

    unsafe fn create_render_texture(
        &mut self,
        handle: RenderTextureHandle,
        params: RenderTextureParams,
    ) -> Result<()> {
        let id = if params.sampler {
            let mut id = 0;
            gl::GenTextures(1, &mut id);
            assert!(id != 0);

            self.bind_texture(0, id)?;
            self.update_texture_params(id, params.wrap, params.filter, 1)?;

            let (internal_format, format, pixel_type) = params.format.into();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as GLint,
                params.dimensions.x as GLsizei,
                params.dimensions.y as GLsizei,
                0,
                format,
                pixel_type,
                ::std::ptr::null(),
            );

            id
        } else {
            let mut id = 0;
            gl::GenRenderbuffers(1, &mut id);
            assert!(id != 0);

            self.bind_render_buffer(id)?;

            let (internal_format, _, _) = params.format.into();
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                internal_format,
                params.dimensions.x as GLint,
                params.dimensions.y as GLint,
            );
            id
        };

        check()?;

        self.render_textures.create(
            handle,
            GLRenderTexture {
                id: id,
                params: params,
            },
        );

        Ok(())
    }

    unsafe fn delete_render_texture(&mut self, handle: RenderTextureHandle) -> Result<()> {
        let rt = self.render_textures
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if rt.params.sampler {
            self.delete_texture_intern(rt.id)
        } else {
            let mut mutables = self.mutables.borrow_mut();
            if mutables.binded_render_buffer == Some(rt.id) {
                mutables.binded_render_buffer = None;
            }

            gl::DeleteRenderbuffers(1, &rt.id);
            check()
        }
    }

    unsafe fn create_mesh(
        &mut self,
        handle: MeshHandle,
        params: MeshParams,
        data: Option<MeshData>,
    ) -> Result<()> {
        let vbo = self.create_buffer_intern(
            gl::ARRAY_BUFFER,
            params.hint,
            params.vertex_buffer_len(),
            data.as_ref().map(|v| v.vptr.as_ref()),
        )?;

        let ibo = self.create_buffer_intern(
            gl::ELEMENT_ARRAY_BUFFER,
            params.hint,
            params.index_buffer_len(),
            data.as_ref().map(|v| v.iptr.as_ref()),
        )?;

        self.meshes.create(
            handle,
            GLMesh {
                vbo: vbo,
                ibo: ibo,
                params: params,
            },
        );

        Ok(())
    }

    unsafe fn update_vertex_buffer(
        &mut self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        let vbo = {
            let mesh = self.meshes
                .get(handle)
                .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

            if mesh.params.hint == MeshHint::Immutable {
                bail!("Trying to update immutable buffer");
            }

            mesh.vbo
        };

        self.update_buffer_intern(gl::ARRAY_BUFFER, vbo, offset, data)?;
        Ok(())
    }

    unsafe fn update_index_buffer(
        &mut self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        let ibo = {
            let mesh = self.meshes
                .get(handle)
                .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

            if mesh.params.hint == MeshHint::Immutable {
                bail!("Trying to update immutable buffer");
            }

            mesh.ibo
        };

        self.update_buffer_intern(gl::ELEMENT_ARRAY_BUFFER, ibo, offset, data)?;
        Ok(())
    }

    unsafe fn delete_mesh(&mut self, handle: MeshHandle) -> Result<()> {
        let mesh = self.meshes
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        // Removes deprecated `VertexArrayObject`s.
        self.mutables
            .borrow_mut()
            .vaos
            .retain(|&(_, vbo), _| vbo != mesh.vbo);

        self.delete_buffer_intern(gl::ARRAY_BUFFER, mesh.vbo)?;
        self.delete_buffer_intern(gl::ELEMENT_ARRAY_BUFFER, mesh.ibo)?;
        Ok(())
    }

    unsafe fn bind(&mut self, id: SurfaceHandle, dimensions: math::Vector2<u32>) -> Result<()> {
        if self.mutables.borrow().binded_surface == Some(id) {
            return Ok(());
        }

        let surface = self.surfaces
            .get(id)
            .ok_or_else(|| format_err!("{:?} is invalid.", id))?;

        // Bind frame buffer.
        let dimensions = if let Some(ref fbo) = surface.fbo {
            self.bind_framebuffer(fbo.id, true)?;
            fbo.dimensions
        } else {
            self.bind_framebuffer(0, false)?;
            dimensions
        };

        // Reset the viewport and scissor box.
        let vp = SurfaceViewport {
            position: math::Vector2::new(0, 0),
            size: dimensions,
        };

        self.set_viewport(vp)?;
        self.set_scissor(SurfaceScissor::Disable)?;

        if !self.mutables.borrow().binded_frame_surfaces.contains(&id) {
            // Sets depth write enable to make sure that we can clear depth buffer properly.
            if surface.params.clear_depth.is_some() {
                self.set_depth_test(true, Comparison::Always)?;
            }

            // Clears frame buffer.
            self.clear(
                surface.params.clear_color,
                surface.params.clear_depth,
                surface.params.clear_stencil,
            )?;

            self.mutables.borrow_mut().binded_frame_surfaces.insert(id);
        }

        self.mutables.borrow_mut().binded_surface = Some(id);
        Ok(())
    }

    unsafe fn update_surface_scissor(&mut self, scissor: SurfaceScissor) -> Result<()> {
        self.set_scissor(scissor)
    }

    unsafe fn update_surface_viewport(&mut self, vp: SurfaceViewport) -> Result<()> {
        self.set_viewport(vp)
    }

    unsafe fn draw(
        &mut self,
        shader: ShaderHandle,
        mesh: MeshHandle,
        mesh_index: MeshIndex,
        uniforms: &[UniformVar],
    ) -> Result<u32> {
        let mesh = {
            // Bind program and associated uniforms and textures.
            let shader = self.shaders
                .get(shader)
                .ok_or_else(|| format_err!("{:?} is invalid.", shader))?;

            self.bind_shader(&shader)?;
            self.clear_binded_texture()?;

            let mut index = 0usize;
            for &(field, variable) in uniforms {
                let location = shader.hash_uniform_location(field).unwrap();
                match variable {
                    UniformVariable::Texture(handle) => {
                        let v = UniformVariable::I32(index as i32);
                        let texture = self.textures.get(handle).map(|v| v.id).unwrap_or(0);
                        self.bind_uniform_variable(location, &v)?;
                        self.bind_texture(index, texture)?;
                        index += 1;
                    }
                    UniformVariable::RenderTexture(handle) => {
                        let v = UniformVariable::I32(index as i32);
                        self.bind_uniform_variable(location, &v)?;

                        if let Some(texture) = self.render_textures.get(handle) {
                            if !texture.params.sampler {
                                bail!("The render buffer does not have a sampler.");
                            }

                            self.bind_texture(index, texture.id)?;
                        } else {
                            self.bind_texture(index, 0)?;
                        }

                        index += 1;
                    }
                    _ => {
                        self.bind_uniform_variable(location, &variable)?;
                    }
                }
            }

            // Bind vertex buffer and vertex array object.
            let mesh = self.meshes
                .get(mesh)
                .ok_or_else(|| format_err!("{:?} is invalid.", mesh))?;

            self.bind_buffer(gl::ARRAY_BUFFER, mesh.vbo)?;
            self.bind_vao(&shader, &mesh)?;
            mesh
        };

        // Bind index buffer object if available.
        self.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, mesh.ibo)?;

        let (from, len) = match mesh_index {
            MeshIndex::Ptr(from, len) => {
                if (from + len) > mesh.params.num_idxes {
                    bail!("MeshIndex is out of bounds");
                }

                ((from * mesh.params.index_format.stride()), len)
            }
            MeshIndex::SubMesh(index) => {
                let num = mesh.params.sub_mesh_offsets.len();
                let from = mesh.params
                    .sub_mesh_offsets
                    .get(index)
                    .ok_or_else(|| format_err!("MeshIndex is out of bounds"))?;

                let to = if index == (num - 1) {
                    mesh.params.num_idxes
                } else {
                    mesh.params.sub_mesh_offsets[index + 1]
                };

                ((from * mesh.params.index_format.stride()), (to - from))
            }
            MeshIndex::All => (0, mesh.params.num_idxes),
        };

        gl::DrawElements(
            mesh.params.primitive.into(),
            len as i32,
            mesh.params.index_format.into(),
            from as *const u32 as *const ::std::os::raw::c_void,
        );

        check()?;
        Ok(mesh.params.primitive.assemble(len as u32))
    }

    unsafe fn flush(&mut self) -> Result<()> {
        gl::Finish();
        check()
    }
}

impl GLVisitor {
    unsafe fn bind_framebuffer(&self, id: GLuint, check_status: bool) -> Result<()> {
        if self.mutables.borrow().binded_framebuffer == Some(id) {
            return Ok(());
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, id);

        if check_status && gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                self.mutables.borrow_mut().binded_framebuffer = Some(0);

                match status {
                    gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
                        bail!("[GL] Surface is incomplete. Not all framebuffer attachment points \
                        are framebuffer attachment complete. This means that at least one attachment point with a \
                        renderbuffer or texture attached has its attached object no longer in existence or has an \
                        attached image with a width or height of zero, or the color attachment point has a non-color-renderable \
                        image attached, or the depth attachment point has a non-depth-renderable image attached, or \
                        the stencil attachment point has a non-stencil-renderable image attached. ");
                    }

                    gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                        bail!("[GL] Surface is incomplete. No images are attached to the framebuffer.");
                    }

                    gl::FRAMEBUFFER_UNSUPPORTED => {
                        bail!("[GL] Surface is incomplete. The combination of internal formats \
                        of the attached images violates an implementation-dependent set of restrictions. ");
                    }

                    _ => {
                        bail!("[GL] Surface is incomplete.");
                    }
                }
            }
        } else {
            self.mutables.borrow_mut().binded_framebuffer = Some(id);
        }

        check()
    }

    unsafe fn bind_shader(&self, shader: &GLShader) -> Result<()> {
        if self.mutables.borrow().binded_shader == Some(shader.id) {
            return Ok(());
        }

        gl::UseProgram(shader.id);
        check()?;

        let rs = shader.params.state;
        self.set_cull_face(rs.cull_face)?;
        self.set_front_face_order(rs.front_face_order)?;
        self.set_depth_test(rs.depth_write, rs.depth_test)?;
        self.set_depth_write_offset(rs.depth_write_offset)?;
        self.set_color_blend(rs.color_blend)?;
        self.set_color_write(rs.color_write)?;

        self.mutables.borrow_mut().binded_shader = Some(shader.id);
        Ok(())
    }

    unsafe fn bind_uniform_variable(
        &self,
        location: GLint,
        variable: &UniformVariable,
    ) -> Result<()> {
        match *variable {
            UniformVariable::Texture(_) => unreachable!(),
            UniformVariable::RenderTexture(_) => unreachable!(),
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

    unsafe fn bind_buffer(&self, tp: GLuint, id: GLuint) -> Result<()> {
        assert!(tp == gl::ARRAY_BUFFER || tp == gl::ELEMENT_ARRAY_BUFFER);
        gl::BindBuffer(tp, id);
        self.mutables.borrow_mut().binded_buffers.insert(tp, id);
        check()
    }

    unsafe fn bind_texture(&self, index: usize, id: GLuint) -> Result<()> {
        // assert!(id != 0, "failed to bind texture with 0.");

        if index >= MAX_UNIFORM_TEXTURE_SLOTS {
            bail!("Reaching maximum texture slots.");
        }

        let mut mutables = self.mutables.borrow_mut();
        if mutables.binded_texture_index != index {
            mutables.binded_texture_index = index;
            gl::ActiveTexture(gl::TEXTURE0 + index as GLuint);
        }

        if mutables.binded_textures[index] != Some(id) {
            mutables.binded_textures[index] = Some(id);
            gl::BindTexture(gl::TEXTURE_2D, id);
        }

        check()
    }

    unsafe fn clear_binded_texture(&self) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();

        for (i, v) in mutables.binded_textures.iter_mut().enumerate() {
            if v.is_some() {
                gl::ActiveTexture(gl::TEXTURE0 + i as GLuint);
                gl::BindTexture(gl::TEXTURE_2D, 0);

                *v = None;
            }
        }

        check()
    }

    unsafe fn bind_render_buffer(&self, rb: GLuint) -> Result<()> {
        assert!(rb != 0, "failed to bind render buffer with 0.");

        let mut mutables = self.mutables.borrow_mut();
        if mutables.binded_render_buffer == Some(rb) {
            return Ok(());
        }

        gl::BindRenderbuffer(gl::RENDERBUFFER, rb);
        mutables.binded_render_buffer = Some(rb);
        check()
    }

    unsafe fn bind_vao(&self, shader: &GLShader, mesh: &GLMesh) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();
        assert!(mutables.binded_shader == Some(shader.id));
        assert!(*mutables.binded_buffers.get(&gl::ARRAY_BUFFER).unwrap() == mesh.vbo);

        if let Some(vao) = mutables.vaos.get(&(shader.id, mesh.vbo)).cloned() {
            if mutables.binded_vao == Some(vao) {
                return Ok(());
            }

            gl::BindVertexArray(vao);
            mutables.binded_vao = Some(vao);
            return check();
        }

        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        mutables.binded_vao = Some(vao);

        for (name, size, required) in shader.params.attributes.iter() {
            if let Some(element) = mesh.params.layout.element(name) {
                if element.size < size {
                    bail!(
                        "Vertex buffer has incompatible attribute `{:?}` [{:?} - {:?}].",
                        name,
                        element.size,
                        size
                    );
                }

                let offset = mesh.params.layout.offset(name).unwrap();
                let stride = mesh.params.layout.stride();

                let location = shader.attribute_location(name.into())?;
                gl::EnableVertexAttribArray(location as GLuint);
                gl::VertexAttribPointer(
                    location as GLuint,
                    GLsizei::from(element.size),
                    element.format.into(),
                    element.normalized as u8,
                    GLsizei::from(stride),
                    offset as *const u8 as *const ::std::os::raw::c_void,
                );
            } else {
                if required {
                    bail!(
                        "Can't find attribute {:?} description in vertex buffer.",
                        name
                    );
                }
            }
        }

        check()?;

        mutables.vaos.insert((shader.id, mesh.vbo), vao);
        Ok(())
    }
}

impl GLVisitor {
    unsafe fn reset_render_state(&self) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();

        gl::Disable(gl::CULL_FACE);
        mutables.render_state.cull_face = CullFace::Nothing;

        gl::FrontFace(gl::CCW);
        mutables.render_state.front_face_order = FrontFaceOrder::CounterClockwise;

        gl::Disable(gl::DEPTH_TEST);
        gl::DepthMask(gl::FALSE);
        mutables.render_state.depth_write = false;
        gl::DepthFunc(gl::ALWAYS);
        mutables.render_state.depth_test = Comparison::Always;
        gl::Disable(gl::POLYGON_OFFSET_FILL);
        mutables.render_state.depth_write_offset = None;

        gl::Disable(gl::BLEND);
        mutables.render_state.color_blend = None;

        gl::ColorMask(1, 1, 1, 1);
        mutables.render_state.color_write = (true, true, true, true);

        gl::Disable(gl::SCISSOR_TEST);
        mutables.scissor = SurfaceScissor::Disable;

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        check()
    }

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_cull_face(&self, face: CullFace) -> Result<()> {
        let state = &mut self.mutables.borrow_mut().render_state;

        if state.cull_face != face {
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

            state.cull_face = face;
            check()?;
        }

        Ok(())
    }

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face_order(&self, front: FrontFaceOrder) -> Result<()> {
        let state = &mut self.mutables.borrow_mut().render_state;

        if state.front_face_order != front {
            gl::FrontFace(match front {
                FrontFaceOrder::Clockwise => gl::CW,
                FrontFaceOrder::CounterClockwise => gl::CCW,
            });

            state.front_face_order = front;
            check()?;
        }

        Ok(())
    }

    /// Enable or disable writing into the depth buffer and specify the value used for depth
    /// buffer comparisons.
    unsafe fn set_depth_test(&self, write: bool, comparsion: Comparison) -> Result<()> {
        let state = &mut self.mutables.borrow_mut().render_state;

        // Note that even if the depth buffer exists and the depth mask is non-zero,
        // the depth buffer is not updated if the depth test is disabled.
        let enable = comparsion != Comparison::Always || write;
        let last_enable = state.depth_test != Comparison::Always || state.depth_write;
        if enable != last_enable {
            if enable {
                gl::Enable(gl::DEPTH_TEST);
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }
        }

        if state.depth_write != write {
            if write {
                gl::DepthMask(gl::TRUE);
            } else {
                gl::DepthMask(gl::FALSE);
            }

            state.depth_write = write;
        }

        if state.depth_test != comparsion {
            gl::DepthFunc(comparsion.into());
            state.depth_test = comparsion;
        }

        check()
    }

    /// Set `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write_offset(&self, offset: Option<(f32, f32)>) -> Result<()> {
        let state = &mut self.mutables.borrow_mut().render_state;

        if state.depth_write_offset != offset {
            if let Some(v) = offset {
                if v.0 != 0.0 || v.1 != 0.0 {
                    gl::Enable(gl::POLYGON_OFFSET_FILL);
                    gl::PolygonOffset(v.0, v.1);
                } else {
                    gl::Disable(gl::POLYGON_OFFSET_FILL);
                }
            }

            state.depth_write_offset = offset;
            check()?;
        }

        Ok(())
    }

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(
        &self,
        blend: Option<(Equation, BlendFactor, BlendFactor)>,
    ) -> Result<()> {
        let state = &mut self.mutables.borrow_mut().render_state;

        if state.color_blend != blend {
            if let Some((equation, src, dst)) = blend {
                if state.color_blend == None {
                    gl::Enable(gl::BLEND);
                }

                gl::BlendFunc(src.into(), dst.into());
                gl::BlendEquation(equation.into());
            } else if state.color_blend != None {
                gl::Disable(gl::BLEND);
            }

            state.color_blend = blend;
            check()?;
        }

        Ok(())
    }

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(&self, mask: (bool, bool, bool, bool)) -> Result<()> {
        let state = &mut self.mutables.borrow_mut().render_state;

        if state.color_write != mask {
            state.color_write = mask;
            gl::ColorMask(mask.0 as u8, mask.1 as u8, mask.2 as u8, mask.3 as u8);
            check()?;
        }

        Ok(())
    }

    /// Set the scissor box relative to the top-lef corner of th window, in pixels.
    unsafe fn set_scissor(&self, scissor: SurfaceScissor) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();

        match scissor {
            SurfaceScissor::Disable => if mutables.scissor != SurfaceScissor::Disable {
                gl::Disable(gl::SCISSOR_TEST);
            },
            SurfaceScissor::Enable { position, size } => {
                if mutables.scissor == SurfaceScissor::Disable {
                    gl::Enable(gl::SCISSOR_TEST);
                }

                gl::Scissor(
                    GLint::from(position.x),
                    GLint::from(position.y),
                    size.x as i32,
                    size.y as i32,
                );
            }
        }

        mutables.scissor = scissor;
        check()
    }

    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    unsafe fn set_viewport(&self, vp: SurfaceViewport) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();

        if mutables.view != vp {
            gl::Viewport(
                GLint::from(vp.position.x),
                GLint::from(vp.position.y),
                vp.size.x as i32,
                vp.size.y as i32,
            );

            mutables.view = vp;
            check()?;
        }

        Ok(())
    }

    unsafe fn clear<C, D, S>(&self, color: C, depth: D, stencil: S) -> Result<()>
    where
        C: Into<Option<math::Color<f32>>>,
        D: Into<Option<f32>>,
        S: Into<Option<i32>>,
    {
        let color = color.into();
        let depth = depth.into();
        let stencil = stencil.into();

        let mut bits = 0;
        if let Some(v) = color {
            bits |= gl::COLOR_BUFFER_BIT;
            gl::ClearColor(v.r, v.g, v.b, v.a);
        }

        if let Some(v) = depth {
            bits |= gl::DEPTH_BUFFER_BIT;
            gl::ClearDepth(f64::from(v));
        }

        if let Some(v) = stencil {
            bits |= gl::STENCIL_BUFFER_BIT;
            gl::ClearStencil(v);
        }

        if bits != 0 {
            gl::Clear(bits);
            check()
        } else {
            Ok(())
        }
    }
}

impl GLVisitor {
    unsafe fn update_framebuffer_render_texture(
        &self,
        id: GLuint,
        params: RenderTextureParams,
        index: usize,
    ) -> Result<()> {
        let mutables = self.mutables.borrow();
        assert!(mutables.binded_framebuffer.is_some() && mutables.binded_framebuffer != Some(0));

        match params.format {
            RenderTextureFormat::RGB8 | RenderTextureFormat::RGBA4 | RenderTextureFormat::RGBA8 => {
                let location = gl::COLOR_ATTACHMENT0 + index as u32;

                if params.sampler {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_2D,
                        id,
                        0,
                    );
                } else {
                    gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, location, gl::RENDERBUFFER, id);
                }
            }
            RenderTextureFormat::Depth16
            | RenderTextureFormat::Depth24
            | RenderTextureFormat::Depth32 => if params.sampler {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_ATTACHMENT,
                    gl::TEXTURE_2D,
                    id,
                    0,
                );
            } else {
                gl::FramebufferRenderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_ATTACHMENT,
                    gl::RENDERBUFFER,
                    id,
                );
            },
            RenderTextureFormat::Depth24Stencil8 => if params.sampler {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::TEXTURE_2D,
                    id,
                    0,
                );
            } else {
                gl::FramebufferRenderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::RENDERBUFFER,
                    id,
                );
            },
        }

        check()
    }

    unsafe fn compile(&self, shader: GLenum, src: &str) -> Result<GLuint> {
        let shader = gl::CreateShader(shader);
        // Attempt to compile the shader
        let c_str = ::std::ffi::CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ::std::ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = GLint::from(gl::FALSE);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != GLint::from(gl::TRUE) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ::std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            bail!("{:?}\n{:?}", ::std::str::from_utf8(&buf).unwrap(), src);
        } else {
            Ok(shader)
        }
    }

    unsafe fn link(&self, vs: GLuint, fs: GLuint) -> Result<GLuint> {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);

        gl::LinkProgram(program);
        // Get the link status
        let mut status = GLint::from(gl::FALSE);
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != GLint::from(gl::TRUE) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ::std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            bail!("{:?}", ::std::str::from_utf8(&buf).unwrap());
        } else {
            Ok(program)
        }
    }

    unsafe fn delete_shader_intern(&mut self, id: GLuint) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();
        if mutables.binded_shader == Some(id) {
            mutables.binded_shader = None;
        }

        gl::DeleteProgram(id);
        check()
    }

    unsafe fn create_buffer_intern(
        &mut self,
        tp: GLuint,
        hint: MeshHint,
        size: usize,
        data: Option<&[u8]>,
    ) -> Result<GLuint> {
        let mut id = 0;
        gl::GenBuffers(1, &mut id);
        assert!(id != 0);

        self.bind_buffer(tp, id)?;

        let value = match data {
            Some(v) if !v.is_empty() => &v[0] as *const u8 as *const ::std::os::raw::c_void,
            _ => ::std::ptr::null(),
        };

        gl::BufferData(tp, size as isize, value, hint.into());
        check()?;
        Ok(id)
    }

    unsafe fn update_buffer_intern(
        &mut self,
        tp: GLuint,
        id: GLuint,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        self.bind_buffer(tp, id)?;
        gl::BufferSubData(
            tp,
            offset as isize,
            data.len() as isize,
            &data[0] as *const u8 as *const ::std::os::raw::c_void,
        );
        check()
    }

    unsafe fn delete_buffer_intern(&mut self, tp: GLuint, id: GLuint) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();
        if mutables.binded_buffers.get(&tp) == Some(&id) {
            mutables.binded_buffers.remove(&tp);
        }

        gl::DeleteBuffers(1, &id);
        check()
    }

    unsafe fn update_texture_params(
        &self,
        id: GLuint,
        wrap: TextureWrap,
        filter: TextureFilter,
        levels: u32,
    ) -> Result<GLuint> {
        let wrap: GLenum = wrap.into();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap as GLint);

        match filter {
            TextureFilter::Nearest => {
                let min_filter = if levels > 1 {
                    gl::NEAREST_MIPMAP_NEAREST
                } else {
                    gl::NEAREST
                };

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            }
            TextureFilter::Linear => {
                let min_filter = if levels > 1 {
                    gl::LINEAR_MIPMAP_LINEAR
                } else {
                    gl::LINEAR
                };

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            }
        }

        if levels > 1 {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, (levels - 1) as GLint);
        }

        Ok(id)
    }

    unsafe fn delete_texture_intern(&mut self, id: GLuint) -> Result<()> {
        let mut mutables = self.mutables.borrow_mut();

        for v in mutables.binded_textures.iter_mut() {
            if *v == Some(id) {
                *v = None;
            }
        }

        gl::DeleteTextures(1, &id);
        check()
    }
}

unsafe fn check_capabilities(caps: &Capabilities) -> Result<()> {
    if caps.version < Version::GL(1, 5) && caps.version < Version::ES(2, 0)
        && (!caps.extensions.gl_arb_vertex_buffer_object
            || !caps.extensions.gl_arb_map_buffer_range)
    {
        bail!("The OpenGL implementation does not supports vertex buffer objects.");
    }

    if caps.version < Version::GL(2, 0) && caps.version < Version::ES(2, 0)
        && (!caps.extensions.gl_arb_shader_objects
            || !caps.extensions.gl_arb_vertex_shader
            || !caps.extensions.gl_arb_fragment_shader)
    {
        bail!("The OpenGL implementation does not supports shader objects.");
    }

    if caps.version < Version::GL(3, 0)
        && caps.version < Version::ES(2, 0)
        && !caps.extensions.gl_ext_framebuffer_object
        && !caps.extensions.gl_arb_framebuffer_object
    {
        bail!("The OpenGL implementation does not supports framebuffer objects.");
    }

    if caps.version < Version::ES(2, 0)
        && caps.version < Version::GL(3, 0)
        && !caps.extensions.gl_ext_framebuffer_blit
    {
        bail!("The OpenGL implementation does not supports blitting framebuffer.");
    }

    if caps.version < Version::GL(3, 1)
        && caps.version < Version::ES(3, 0)
        && !caps.extensions.gl_arb_uniform_buffer_object
    {
        bail!("The OpenGL implementation does not supports uniform buffer objects.");
    }

    if caps.version < Version::GL(3, 0)
        && caps.version < Version::ES(3, 0)
        && !caps.extensions.gl_arb_vertex_array_object
        && !caps.extensions.gl_apple_vertex_array_object
        && !caps.extensions.gl_oes_vertex_array_object
    {
        bail!("The OpenGL implementation does not supports vertex array objects.");
    }

    Ok(())
}

unsafe fn check() -> Result<()> {
    match gl::GetError() {
        gl::NO_ERROR => Ok(()),

        gl::INVALID_ENUM => {
            bail!("[GL] An unacceptable value is specified for an enumerated argument.")
        }

        gl::INVALID_VALUE => bail!("[GL] A numeric argument is out of range."),

        gl::INVALID_OPERATION => {
            bail!("[GL] The specified operation is not allowed in the current state.")
        }

        gl::INVALID_FRAMEBUFFER_OPERATION => bail!(
            r"[GL] The command is trying to render to or read from the framebufferwhile the \
            currently bound framebuffer is not framebuffer complete."
        ),

        gl::OUT_OF_MEMORY => bail!("[GL] There is not enough memory left to execute the command."),
        _ => bail!("[GL] Oops, Unknown OpenGL error."),
    }
}
