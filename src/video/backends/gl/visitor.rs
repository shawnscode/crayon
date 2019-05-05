use std::cell::RefCell;

use gl;
use gl::types::*;
use smallvec::SmallVec;

use crate::errors::*;
use crate::math::prelude::{Aabb2, Color, Vector2};
use crate::utils::hash::{FastHashMap, FastHashSet};
use crate::utils::hash_value::HashValue;

use super::super::super::assets::prelude::*;
use super::super::utils::DataVec;
use super::super::{UniformVar, Visitor};
use super::capabilities::{Capabilities, Version};
use super::types;

#[derive(Debug, Clone)]
struct GLSurfaceData {
    handle: SurfaceHandle,
    id: Option<GLuint>,
    dimensions: Option<Vector2<u32>>,
    params: SurfaceParams,
}

#[derive(Debug, Clone)]
struct GLShaderData {
    handle: ShaderHandle,
    id: GLuint,
    params: ShaderParams,
    uniforms: RefCell<FastHashMap<HashValue<str>, GLint>>,
    attributes: RefCell<FastHashMap<HashValue<str>, GLint>>,
}

impl GLShaderData {
    fn hash_uniform_location<T: Into<HashValue<str>>>(&self, name: T) -> Option<GLint> {
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
struct GLMeshData {
    handle: MeshHandle,
    vbo: GLuint,
    ibo: GLuint,
    params: MeshParams,
}

#[derive(Debug, Clone)]
struct GLTextureData {
    handle: TextureHandle,
    id: GLuint,
    params: TextureParams,
    allocated: RefCell<bool>,
}

#[derive(Debug, Copy, Clone)]
struct GLRenderTextureData {
    handle: RenderTextureHandle,
    id: GLuint,
    params: RenderTextureParams,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Sampler {
    RenderTexture(RenderTextureHandle),
    Texture(TextureHandle),
}

struct GLMutableState {
    render_state: RenderState,
    scissor: SurfaceScissor,
    view: SurfaceViewport,
    cleared_surfaces: FastHashSet<SurfaceHandle>,
    vaos: FastHashMap<(ShaderHandle, MeshHandle), GLuint>,
    binded_surface: Option<SurfaceHandle>,
    binded_shader: Option<ShaderHandle>,
    binded_vao: Option<(ShaderHandle, MeshHandle)>,
    binded_texture_index: usize,
    binded_textures: SmallVec<[Option<Sampler>; 8]>,
}

pub struct GLVisitor {
    state: GLMutableState,
    capabilities: Capabilities,
    surfaces: DataVec<GLSurfaceData>,
    shaders: DataVec<GLShaderData>,
    meshes: DataVec<GLMeshData>,
    textures: DataVec<GLTextureData>,
    render_textures: DataVec<GLRenderTextureData>,
}

impl GLVisitor {
    pub unsafe fn new() -> Result<Self> {
        let capabilities = Capabilities::parse()?;
        info!("GLVisitor {:#?}", capabilities);
        check_capabilities(&capabilities)?;

        let state = GLMutableState {
            render_state: RenderState::default(),
            scissor: SurfaceScissor::Disable,
            view: SurfaceViewport {
                position: Vector2::new(0, 0),
                size: Vector2::new(0, 0),
            },
            cleared_surfaces: FastHashSet::default(),
            vaos: FastHashMap::default(),
            binded_surface: None,
            binded_shader: None,
            binded_vao: None,
            binded_texture_index: 0,
            binded_textures: SmallVec::new(),
        };

        let mut visitor = GLVisitor {
            state,
            capabilities,
            surfaces: DataVec::new(),
            shaders: DataVec::new(),
            meshes: DataVec::new(),
            textures: DataVec::new(),
            render_textures: DataVec::new(),
        };

        Self::reset_render_state(&mut visitor.state)?;
        Ok(visitor)
    }
}

impl Visitor for GLVisitor {
    unsafe fn advance(&mut self) -> Result<()> {
        self.state.cleared_surfaces.clear();
        self.state.binded_surface = None;
        Ok(())
    }

    unsafe fn create_surface(
        &mut self,
        handle: SurfaceHandle,
        params: SurfaceParams,
    ) -> Result<()> {
        let mut data = GLSurfaceData {
            handle,
            params,
            id: None,
            dimensions: None,
        };

        if params.colors[0].is_some() || params.depth_stencil.is_some() {
            let mut id = 0;
            gl::GenFramebuffers(1, &mut id);
            assert!(id != 0);

            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
            self.state.binded_surface = None;

            let mut dimensions = None;
            for (i, attachment) in params.colors.iter().enumerate() {
                if let Some(v) = *attachment {
                    let rt = self
                        .render_textures
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
                let rt = self
                    .render_textures
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

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                self.state.binded_surface = None;

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

            data.id = Some(id);
            data.dimensions = dimensions;
        };

        self.surfaces.create(handle, data);

        Ok(())
    }

    unsafe fn delete_surface(&mut self, handle: SurfaceHandle) -> Result<()> {
        let surface = self
            .surfaces
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if self.state.binded_surface == Some(handle) {
            self.state.binded_surface = None;
        }

        if let Some(id) = surface.id {
            gl::DeleteFramebuffers(1, &id);
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
        let vs = Self::compile(gl::VERTEX_SHADER, vs)?;
        let fs = Self::compile(gl::FRAGMENT_SHADER, fs)?;
        let id = Self::link(&[vs, fs])?;

        gl::DetachShader(id, vs);
        gl::DeleteShader(vs);
        gl::DetachShader(id, fs);
        gl::DeleteShader(fs);
        check()?;

        let shader = GLShaderData {
            handle,
            id,
            params,
            uniforms: RefCell::new(FastHashMap::default()),
            attributes: RefCell::new(FastHashMap::default()),
        };

        for (name, _, _) in shader.params.attributes.iter() {
            let name: &'static str = name.into();
            let location = shader.attribute_location(name)?;
            if location == -1 {
                gl::DeleteProgram(id);
                bail!("Attribute({:?}) is undefined in shader sources.", name);
            }
        }

        for &(ref name, _) in shader.params.uniforms.iter() {
            let location = shader.uniform_location(name)?;
            if location == -1 {
                gl::DeleteProgram(id);
                bail!("Uniform({:?}) is undefined in shader sources.", name);
            }
        }

        self.shaders.create(handle, shader);
        Ok(())
    }

    unsafe fn delete_shader(&mut self, handle: ShaderHandle) -> Result<()> {
        let shader = self
            .shaders
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        // Removes deprecated `VertexArrayObject`s.
        self.state.vaos.retain(|&(h, _), vao| {
            if h == shader.handle {
                gl::DeleteVertexArrays(1, vao as *mut u32);
                false
            } else {
                true
            }
        });

        if self.state.binded_shader == Some(handle) {
            self.state.binded_shader = None;
        }

        gl::DeleteProgram(shader.id);
        check()
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

        let (internal_format, format, pixel_type) =
            types::texture_format(params.format, &self.capabilities);
        let compressed = params.format.compressed();
        let mut allocated = false;

        if let Some(mut data) = data {
            let len = data.bytes.len();
            if len > 0 {
                Self::bind_texture(&mut self.state, Some(Sampler::Texture(handle)), 0, id)?;
                Self::bind_texture_params(params.wrap, params.filter, len as u32)?;

                let mut dims = (
                    params.dimensions.x as GLsizei,
                    params.dimensions.y as GLsizei,
                );

                if compressed {
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
            GLTextureData {
                handle,
                id,
                params,
                allocated: RefCell::new(allocated),
            },
        );

        Ok(())
    }

    unsafe fn update_texture(
        &mut self,
        handle: TextureHandle,
        area: Aabb2<u32>,
        data: &[u8],
    ) -> Result<()> {
        let texture = self
            .textures
            .get(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if texture.params.hint == TextureHint::Immutable {
            bail!("Trying to update immutable texture.");
        }

        if texture.params.format.compressed() {
            bail!("Trying to update compressed texture.");
        }

        if data.len() > texture.params.format.size(area.dim()) as usize
            || area.min.x >= texture.params.dimensions.x
            || area.min.y >= texture.params.dimensions.y
        {
            bail!("Trying to update texture data out of bounds.");
        }

        let (internal_format, format, pixel_type) =
            types::texture_format(texture.params.format, &self.capabilities);

        Self::bind_texture(
            &mut self.state,
            Some(Sampler::Texture(handle)),
            0,
            texture.id,
        )?;

        if !*texture.allocated.borrow() {
            Self::bind_texture_params(texture.params.wrap, texture.params.filter, 1)?;

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

            *texture.allocated.borrow_mut() = true;
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
        let texture = self
            .textures
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        for v in self.state.binded_textures.iter_mut() {
            if *v == Some(Sampler::Texture(handle)) {
                *v = None;
            }
        }

        gl::DeleteTextures(1, &texture.id);
        check()
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

            Self::bind_texture(&mut self.state, Some(Sampler::RenderTexture(handle)), 0, id)?;
            Self::bind_texture_params(params.wrap, params.filter, 1)?;

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
            gl::BindRenderbuffer(gl::RENDERBUFFER, id);

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

        self.render_textures
            .create(handle, GLRenderTextureData { handle, id, params });

        Ok(())
    }

    unsafe fn delete_render_texture(&mut self, handle: RenderTextureHandle) -> Result<()> {
        let rt = self
            .render_textures
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if rt.params.sampler {
            for v in self.state.binded_textures.iter_mut() {
                if *v == Some(Sampler::RenderTexture(handle)) {
                    *v = None;
                }
            }

            gl::DeleteTextures(1, &rt.id);
        } else {
            gl::DeleteRenderbuffers(1, &rt.id);
        }

        check()
    }

    unsafe fn create_mesh(
        &mut self,
        handle: MeshHandle,
        params: MeshParams,
        data: Option<MeshData>,
    ) -> Result<()> {
        let vbo = self.create_buffer(
            gl::ARRAY_BUFFER,
            params.hint,
            params.vertex_buffer_len(),
            data.as_ref().map(|v| v.vptr.as_ref()),
        )?;

        let ibo = self.create_buffer(
            gl::ELEMENT_ARRAY_BUFFER,
            params.hint,
            params.index_buffer_len(),
            data.as_ref().map(|v| v.iptr.as_ref()),
        )?;

        self.meshes.create(
            handle,
            GLMeshData {
                handle,
                vbo,
                ibo,
                params,
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
            let mesh = self
                .meshes
                .get(handle)
                .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

            if mesh.params.hint == MeshHint::Immutable {
                bail!("Trying to update immutable buffer");
            }

            mesh.vbo
        };

        Self::update_buffer(gl::ARRAY_BUFFER, vbo, offset, data)?;
        Ok(())
    }

    unsafe fn update_index_buffer(
        &mut self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        let ibo = {
            let mesh = self
                .meshes
                .get(handle)
                .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

            if mesh.params.hint == MeshHint::Immutable {
                bail!("Trying to update immutable buffer");
            }

            mesh.ibo
        };

        Self::update_buffer(gl::ELEMENT_ARRAY_BUFFER, ibo, offset, data)?;
        Ok(())
    }

    unsafe fn delete_mesh(&mut self, handle: MeshHandle) -> Result<()> {
        let mesh = self
            .meshes
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        // Removes deprecated `VertexArrayObject`s.
        self.state.vaos.retain(|&(_, h), vao| {
            if h == mesh.handle {
                gl::DeleteVertexArrays(1, vao as *mut u32);
                false
            } else {
                true
            }
        });

        gl::DeleteBuffers(1, &mesh.vbo);
        gl::DeleteBuffers(1, &mesh.ibo);
        check()
    }

    unsafe fn bind(&mut self, handle: SurfaceHandle, dimensions: Vector2<u32>) -> Result<()> {
        if self.state.binded_surface == Some(handle) {
            return Ok(());
        }

        let surface = self
            .surfaces
            .get(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        // Bind frame buffer.
        let id = surface.id.unwrap_or(0);
        let dimensions = surface.dimensions.unwrap_or(dimensions);
        gl::BindFramebuffer(gl::FRAMEBUFFER, id);

        // Reset the viewport and scissor box.
        let vp = SurfaceViewport {
            position: Vector2::new(0, 0),
            size: dimensions,
        };

        Self::set_viewport(&mut self.state, vp)?;
        Self::set_scissor(&mut self.state, SurfaceScissor::Disable)?;

        if !self.state.cleared_surfaces.contains(&handle) {
            // Sets depth write enable to make sure that we can clear depth buffer properly.
            if surface.params.clear_depth.is_some() {
                self.state.binded_shader = None;
                Self::set_depth_test(&mut self.state, true, Comparison::Always)?;
            }

            // Clears frame buffer.
            Self::clear(
                surface.params.clear_color,
                surface.params.clear_depth,
                surface.params.clear_stencil,
            )?;

            self.state.cleared_surfaces.insert(handle);
        }

        self.state.binded_surface = Some(handle);
        Ok(())
    }

    unsafe fn update_surface_scissor(&mut self, scissor: SurfaceScissor) -> Result<()> {
        Self::set_scissor(&mut self.state, scissor)
    }

    unsafe fn update_surface_viewport(&mut self, vp: SurfaceViewport) -> Result<()> {
        Self::set_viewport(&mut self.state, vp)
    }

    unsafe fn draw(
        &mut self,
        shader: ShaderHandle,
        mesh: MeshHandle,
        mesh_index: MeshIndex,
        uniforms: &[UniformVar],
    ) -> Result<u32> {
        // Bind program and associated uniforms and textures.
        let shader = self
            .shaders
            .get(shader)
            .ok_or_else(|| format_err!("{:?} is invalid.", shader))?;

        Self::bind_shader(&mut self.state, &shader)?;

        let mut index = 0usize;
        for &(field, variable) in uniforms {
            if let Some(tp) = shader.params.uniforms.variable_type(field) {
                if tp != variable.variable_type() {
                    let name = shader.params.uniforms.variable_name(field).unwrap();
                    bail!(
                        "The uniform {} needs a {:?} instead of {:?}.",
                        name,
                        tp,
                        variable.variable_type(),
                    );
                }

                let location = shader.hash_uniform_location(field).unwrap();
                match variable {
                    UniformVariable::Texture(handle) => {
                        let v = UniformVariable::I32(index as i32);
                        Self::bind_uniform_variable(location, &v)?;

                        if let Some(texture) = self.textures.get(handle) {
                            Self::bind_texture(
                                &mut self.state,
                                Some(Sampler::Texture(handle)),
                                index,
                                texture.id,
                            )?;
                        } else {
                            Self::bind_texture(&mut self.state, None, index, 0)?;
                        }

                        index += 1;
                    }
                    UniformVariable::RenderTexture(handle) => {
                        let v = UniformVariable::I32(index as i32);
                        Self::bind_uniform_variable(location, &v)?;

                        if let Some(texture) = self.render_textures.get(handle) {
                            if !texture.params.sampler {
                                bail!("The render buffer does not have a sampler.");
                            }

                            Self::bind_texture(
                                &mut self.state,
                                Some(Sampler::RenderTexture(handle)),
                                index,
                                texture.id,
                            )?;
                        } else {
                            Self::bind_texture(&mut self.state, None, index, 0)?;
                        }

                        index += 1;
                    }
                    _ => {
                        Self::bind_uniform_variable(location, &variable)?;
                    }
                }
            } else {
                bail!("Undefined uniform field {:?}.", field);
            }
        }

        if let Some(mesh) = self.meshes.get(mesh) {
            // Bind vertex buffer and vertex array object.
            Self::bind_mesh(&mut self.state, &shader, &mesh)?;

            let (from, len) = match mesh_index {
                MeshIndex::Ptr(from, len) => {
                    if (from + len) > mesh.params.num_idxes {
                        bail!("MeshIndex is out of bounds");
                    }

                    ((from * mesh.params.index_format.stride()), len)
                }
                MeshIndex::SubMesh(index) => {
                    let num = mesh.params.sub_mesh_offsets.len();
                    let from = mesh
                        .params
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
        } else {
            Ok(0)
        }
    }

    unsafe fn flush(&mut self) -> Result<()> {
        if self.state.cleared_surfaces.is_empty() {
            Self::clear(Color::black(), None, None)?;
        }

        gl::Finish();
        check()
    }
}

impl GLVisitor {
    unsafe fn bind_shader(state: &mut GLMutableState, shader: &GLShaderData) -> Result<()> {
        if state.binded_shader == Some(shader.handle) {
            return Ok(());
        }

        gl::UseProgram(shader.id);
        check()?;

        let rs = shader.params.state;
        Self::set_cull_face(state, rs.cull_face)?;
        Self::set_front_face_order(state, rs.front_face_order)?;
        Self::set_depth_test(state, rs.depth_write, rs.depth_test)?;
        Self::set_depth_write_offset(state, rs.depth_write_offset)?;
        Self::set_color_blend(state, rs.color_blend)?;
        Self::set_color_write(state, rs.color_write)?;

        state.binded_shader = Some(shader.handle);
        Ok(())
    }

    unsafe fn bind_uniform_variable(location: GLint, variable: &UniformVariable) -> Result<()> {
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

    unsafe fn bind_texture(
        state: &mut GLMutableState,
        sampler: Option<Sampler>,
        index: usize,
        id: GLuint,
    ) -> Result<()> {
        if state.binded_texture_index != index {
            state.binded_texture_index = index;
            gl::ActiveTexture(gl::TEXTURE0 + index as GLuint);
        }

        if state.binded_textures.len() <= index {
            state.binded_textures.resize(index + 1, None);
        }

        if state.binded_textures[index] != sampler {
            state.binded_textures[index] = sampler;
            gl::BindTexture(gl::TEXTURE_2D, id);
        }

        check()
    }

    unsafe fn bind_mesh(
        state: &mut GLMutableState,
        shader: &GLShaderData,
        mesh: &GLMeshData,
    ) -> Result<()> {
        assert!(state.binded_shader == Some(shader.handle));

        let k = (shader.handle, mesh.handle);
        if state.binded_vao != Some(k) {
            if let Some(vao) = state.vaos.get(&k).cloned() {
                gl::BindVertexArray(vao);
                check()?;
            } else {
                let mut vao = 0;
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vbo);

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
                    } else if required {
                        bail!(
                            "Can't find attribute {:?} description in vertex buffer.",
                            name
                        );
                    }
                }

                check()?;
                state.vaos.insert(k, vao);
            }

            state.binded_vao = Some(k);
        }

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.ibo);
        Ok(())
    }
}

impl GLVisitor {
    unsafe fn reset_render_state(state: &mut GLMutableState) -> Result<()> {
        gl::Disable(gl::CULL_FACE);
        state.render_state.cull_face = CullFace::Nothing;

        gl::FrontFace(gl::CCW);
        state.render_state.front_face_order = FrontFaceOrder::CounterClockwise;

        gl::Disable(gl::DEPTH_TEST);
        gl::DepthMask(gl::FALSE);
        state.render_state.depth_write = false;
        gl::DepthFunc(gl::ALWAYS);
        state.render_state.depth_test = Comparison::Always;
        gl::Disable(gl::POLYGON_OFFSET_FILL);
        state.render_state.depth_write_offset = None;

        gl::Disable(gl::BLEND);
        state.render_state.color_blend = None;

        gl::ColorMask(1, 1, 1, 1);
        state.render_state.color_write = (true, true, true, true);

        gl::Disable(gl::SCISSOR_TEST);
        state.scissor = SurfaceScissor::Disable;

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        check()
    }

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_cull_face(state: &mut GLMutableState, face: CullFace) -> Result<()> {
        let rs = &mut state.render_state;

        if rs.cull_face != face {
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

            rs.cull_face = face;
            check()?;
        }

        Ok(())
    }

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face_order(
        state: &mut GLMutableState,
        front: FrontFaceOrder,
    ) -> Result<()> {
        let rs = &mut state.render_state;

        if rs.front_face_order != front {
            gl::FrontFace(match front {
                FrontFaceOrder::Clockwise => gl::CW,
                FrontFaceOrder::CounterClockwise => gl::CCW,
            });

            rs.front_face_order = front;
            check()?;
        }

        Ok(())
    }

    /// Enable or disable writing into the depth buffer and specify the value used for depth
    /// buffer comparisons.
    unsafe fn set_depth_test(
        state: &mut GLMutableState,
        write: bool,
        comparsion: Comparison,
    ) -> Result<()> {
        let rs = &mut state.render_state;

        // Note that even if the depth buffer exists and the depth mask is non-zero,
        // the depth buffer is not updated if the depth test is disabled.
        let enable = comparsion != Comparison::Always || write;
        let last_enable = rs.depth_test != Comparison::Always || rs.depth_write;
        if enable != last_enable {
            if enable {
                gl::Enable(gl::DEPTH_TEST);
            } else {
                gl::Disable(gl::DEPTH_TEST);
            }
        }

        if rs.depth_write != write {
            if write {
                gl::DepthMask(gl::TRUE);
            } else {
                gl::DepthMask(gl::FALSE);
            }

            rs.depth_write = write;
        }

        if rs.depth_test != comparsion {
            gl::DepthFunc(comparsion.into());
            rs.depth_test = comparsion;
        }

        check()
    }

    /// Set `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write_offset(
        state: &mut GLMutableState,
        offset: Option<(f32, f32)>,
    ) -> Result<()> {
        let rs = &mut state.render_state;

        if rs.depth_write_offset != offset {
            if let Some(v) = offset {
                if v.0 != 0.0 || v.1 != 0.0 {
                    gl::Enable(gl::POLYGON_OFFSET_FILL);
                    gl::PolygonOffset(v.0, v.1);
                } else {
                    gl::Disable(gl::POLYGON_OFFSET_FILL);
                }
            }

            rs.depth_write_offset = offset;
            check()?;
        }

        Ok(())
    }

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(
        state: &mut GLMutableState,
        blend: Option<(Equation, BlendFactor, BlendFactor)>,
    ) -> Result<()> {
        let rs = &mut state.render_state;

        if rs.color_blend != blend {
            if let Some((equation, src, dst)) = blend {
                if rs.color_blend == None {
                    gl::Enable(gl::BLEND);
                }

                gl::BlendFunc(src.into(), dst.into());
                gl::BlendEquation(equation.into());
            } else if rs.color_blend != None {
                gl::Disable(gl::BLEND);
            }

            rs.color_blend = blend;
            check()?;
        }

        Ok(())
    }

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(
        state: &mut GLMutableState,
        mask: (bool, bool, bool, bool),
    ) -> Result<()> {
        let rs = &mut state.render_state;

        if rs.color_write != mask {
            rs.color_write = mask;
            gl::ColorMask(mask.0 as u8, mask.1 as u8, mask.2 as u8, mask.3 as u8);
            check()?;
        }

        Ok(())
    }

    /// Set the scissor box relative to the top-lef corner of th window, in pixels.
    unsafe fn set_scissor(state: &mut GLMutableState, scissor: SurfaceScissor) -> Result<()> {
        match scissor {
            SurfaceScissor::Disable => {
                if state.scissor != SurfaceScissor::Disable {
                    gl::Disable(gl::SCISSOR_TEST);
                }
            }
            SurfaceScissor::Enable { position, size } => {
                if state.scissor == SurfaceScissor::Disable {
                    gl::Enable(gl::SCISSOR_TEST);
                }

                gl::Scissor(position.x, position.y, size.x as i32, size.y as i32);
            }
        }

        state.scissor = scissor;
        check()
    }

    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    unsafe fn set_viewport(state: &mut GLMutableState, vp: SurfaceViewport) -> Result<()> {
        if state.view != vp {
            gl::Viewport(
                vp.position.x,
                vp.position.y,
                vp.size.x as i32,
                vp.size.y as i32,
            );

            state.view = vp;
            check()?;
        }

        Ok(())
    }

    unsafe fn clear<C, D, S>(color: C, depth: D, stencil: S) -> Result<()>
    where
        C: Into<Option<Color<f32>>>,
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
        match params.format {
            RenderTextureFormat::RGB8 | RenderTextureFormat::RGBA4 | RenderTextureFormat::RGBA8 => {
                let location = gl::COLOR_ATTACHMENT0 + index as u32;

                if params.sampler {
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, location, gl::TEXTURE_2D, id, 0);
                } else {
                    gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, location, gl::RENDERBUFFER, id);
                }
            }
            RenderTextureFormat::Depth16
            | RenderTextureFormat::Depth24
            | RenderTextureFormat::Depth32 => {
                if params.sampler {
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
                }
            }
            RenderTextureFormat::Depth24Stencil8 => {
                if params.sampler {
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
                }
            }
        }

        check()
    }

    unsafe fn compile(shader: GLenum, src: &str) -> Result<GLuint> {
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

    unsafe fn link<'a, T>(shaders: T) -> Result<GLuint>
    where
        T: IntoIterator<Item = &'a GLuint>,
    {
        let program = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(program, *shader)
        }

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

    unsafe fn create_buffer(
        &mut self,
        tp: GLuint,
        hint: MeshHint,
        size: usize,
        data: Option<&[u8]>,
    ) -> Result<GLuint> {
        let mut id = 0;
        gl::GenBuffers(1, &mut id);
        assert!(id != 0);

        gl::BindBuffer(tp, id);

        let value = match data {
            Some(v) if !v.is_empty() => &v[0] as *const u8 as *const ::std::os::raw::c_void,
            _ => ::std::ptr::null(),
        };

        gl::BufferData(tp, size as isize, value, hint.into());
        check()?;
        Ok(id)
    }

    unsafe fn update_buffer(tp: GLuint, id: GLuint, offset: usize, data: &[u8]) -> Result<()> {
        gl::BindBuffer(tp, id);
        gl::BufferSubData(
            tp,
            offset as isize,
            data.len() as isize,
            &data[0] as *const u8 as *const ::std::os::raw::c_void,
        );
        check()
    }

    unsafe fn bind_texture_params(
        wrap: TextureWrap,
        filter: TextureFilter,
        levels: u32,
    ) -> Result<()> {
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

        Ok(())
    }
}

unsafe fn check_capabilities(caps: &Capabilities) -> Result<()> {
    if caps.version < Version::GL(1, 5)
        && caps.version < Version::ES(2, 0)
        && (!caps.extensions.gl_arb_vertex_buffer_object
            || !caps.extensions.gl_arb_map_buffer_range)
    {
        bail!("The OpenGL implementation does not supports vertex buffer objects.");
    }

    if caps.version < Version::GL(2, 0)
        && caps.version < Version::ES(2, 0)
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
