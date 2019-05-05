use std::cell::RefCell;

use smallvec::SmallVec;
use web_sys::{
    self, HtmlCanvasElement, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlRenderbuffer,
    WebGlShader, WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject,
};

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as WebGL;

use crate::errors::*;
use crate::math::prelude::*;
use crate::utils::hash::{FastHashMap, FastHashSet};
use crate::utils::hash_value::HashValue;
use crate::video::assets::prelude::*;

use super::super::utils::DataVec;
use super::super::{UniformVar, Visitor};
use super::capabilities::Capabilities;

#[derive(Debug, Clone)]
struct GLSurfaceData {
    handle: SurfaceHandle,
    id: Option<WebGlFramebuffer>,
    dims: Option<Vector2<u32>>,
    params: SurfaceParams,
}

#[derive(Debug, Clone)]
pub struct GLShaderData {
    handle: ShaderHandle,
    id: WebGlProgram,
    params: ShaderParams,
    uniforms: RefCell<FastHashMap<HashValue<str>, WebGlUniformLocation>>,
    attributes: RefCell<FastHashMap<HashValue<str>, i32>>,
}

impl GLShaderData {
    fn hash_uniform_location<T: Into<HashValue<str>>>(
        &self,
        name: T,
    ) -> Option<WebGlUniformLocation> {
        self.uniforms.borrow().get(&name.into()).cloned()
    }

    unsafe fn uniform_location(&self, ctx: &WebGL, name: &str) -> Result<WebGlUniformLocation> {
        let hash = name.into();
        let mut uniforms = self.uniforms.borrow_mut();
        match uniforms.get(&hash).cloned() {
            Some(location) => Ok(location),
            None => {
                if let Some(location) = ctx.get_uniform_location(&self.id, name) {
                    check(ctx)?;
                    uniforms.insert(hash, location.clone());
                    Ok(location)
                } else {
                    bail!("Uniform({:?}) is undefined in shader sources.", name);
                }
            }
        }
    }

    unsafe fn attribute_location(&self, ctx: &WebGL, name: &str) -> Result<i32> {
        let hash = name.into();
        let mut attributes = self.attributes.borrow_mut();
        match attributes.get(&hash).cloned() {
            Some(location) => Ok(location),
            None => {
                let location = ctx.get_attrib_location(&self.id, name);
                if location >= 0 {
                    check(ctx).unwrap();
                    attributes.insert(hash, location);
                    Ok(location)
                } else {
                    bail!("Attribute({:?}) is undefined in shader sources.", name);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct GLTextureData {
    handle: TextureHandle,
    id: WebGlTexture,
    params: TextureParams,
    allocated: RefCell<bool>,
}

#[derive(Debug, Clone)]
enum GLRenderTexture {
    R(WebGlRenderbuffer),
    T(WebGlTexture),
}

#[derive(Debug, Clone)]
struct GLRenderTextureData {
    handle: RenderTextureHandle,
    id: GLRenderTexture,
    params: RenderTextureParams,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Sampler {
    RenderTexture(RenderTextureHandle),
    Texture(TextureHandle),
}

#[derive(Debug, Clone)]
struct GLMeshData {
    handle: MeshHandle,
    vbo: WebGlBuffer,
    ibo: WebGlBuffer,
    params: MeshParams,
}

struct WebGLState {
    render_state: RenderState,
    scissor: SurfaceScissor,
    view: SurfaceViewport,
    cleared_surfaces: FastHashSet<SurfaceHandle>,
    vaos: FastHashMap<(ShaderHandle, MeshHandle), WebGlVertexArrayObject>,
    binded_surface: Option<SurfaceHandle>,
    binded_shader: Option<ShaderHandle>,
    binded_texture_index: usize,
    binded_textures: SmallVec<[Option<Sampler>; 8]>,
    binded_vao: Option<(ShaderHandle, MeshHandle)>,
}

pub struct WebGLVisitor {
    ctx: WebGL,
    state: WebGLState,

    capabilities: Capabilities,
    surfaces: DataVec<GLSurfaceData>,
    shaders: DataVec<GLShaderData>,
    meshes: DataVec<GLMeshData>,
    textures: DataVec<GLTextureData>,
    render_textures: DataVec<GLRenderTextureData>,
}

impl WebGLVisitor {
    pub unsafe fn new() -> Result<Self> {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");

        let ctx = document
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap()
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGL>()
            .unwrap();

        let mut state = WebGLState {
            render_state: RenderState::default(),
            scissor: SurfaceScissor::Disable,
            view: SurfaceViewport {
                position: Vector2::new(0, 0),
                size: Vector2::new(0, 0),
            },
            cleared_surfaces: FastHashSet::default(),
            binded_surface: None,
            binded_shader: None,
            binded_texture_index: 0,
            binded_textures: SmallVec::new(),
            vaos: FastHashMap::default(),
            binded_vao: None,
        };

        Self::reset_render_state(&ctx, &mut state)?;

        Ok(WebGLVisitor {
            capabilities: Capabilities::new(&ctx)?,
            ctx: ctx,
            state: state,
            surfaces: DataVec::new(),
            shaders: DataVec::new(),
            textures: DataVec::new(),
            render_textures: DataVec::new(),
            meshes: DataVec::new(),
        })
    }
}

impl Visitor for WebGLVisitor {
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
            handle: handle,
            id: None,
            dims: None,
            params: params,
        };

        if params.colors[0].is_some() || params.depth_stencil.is_some() {
            let id = self.ctx.create_framebuffer().unwrap();
            self.ctx.bind_framebuffer(WebGL::FRAMEBUFFER, Some(&id));
            self.state.binded_surface = None;

            let mut dimensions = None;
            for (i, attachment) in params.colors.iter().enumerate() {
                if let Some(v) = *attachment {
                    let rt = self
                        .render_textures
                        .get(v)
                        .ok_or_else(|| format_err!("RenderTexture handle {:?} is invalid.", v))?;

                    if !rt.params.format.is_color() {
                        bail!("Incompitable(mismatch format) attachments of SurfaceObject.");
                    }

                    if dimensions.is_some() && dimensions != Some(rt.params.dimensions) {
                        bail!("Incompitable(mismatch dimensons) attachments of SurfaceObject.");
                    }

                    dimensions = Some(rt.params.dimensions);
                    Self::bind_surface_render_texture(&self.ctx, &rt, i)?;
                }
            }

            if let Some(v) = params.depth_stencil {
                let rt = self
                    .render_textures
                    .get(v)
                    .ok_or_else(|| format_err!("RenderTexture handle {:?} is invalid.", v))?;

                if rt.params.format.is_color() {
                    bail!("Incompitable(mismatch format) attachments of SurfaceObject.");
                }

                if dimensions.is_some() && dimensions != Some(rt.params.dimensions) {
                    bail!("Incompitable(mismatch dimensions) attachments of SurfaceObject.");
                }

                dimensions = Some(rt.params.dimensions);
                Self::bind_surface_render_texture(&self.ctx, &rt, 0)?;
            }

            let status = self.ctx.check_framebuffer_status(WebGL::FRAMEBUFFER);
            if status != WebGL::FRAMEBUFFER_COMPLETE {
                self.ctx.bind_framebuffer(WebGL::FRAMEBUFFER, None);
                self.state.binded_surface = None;

                match status {
                    WebGL::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
                        bail!("[GL] Surface is incomplete. Not all framebuffer attachment points \
                        are framebuffer attachment complete. This means that at least one attachment point with a \
                        renderbuffer or texture attached has its attached object no longer in existence or has an \
                        attached image with a width or height of zero, or the color attachment point has a non-color-renderable \
                        image attached, or the depth attachment point has a non-depth-renderable image attached, or \
                        the stencil attachment point has a non-stencil-renderable image attached.");
                    }

                    WebGL::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                        bail!("[GL] Surface is incomplete. No images are attached to the framebuffer.");
                    }

                    WebGL::FRAMEBUFFER_UNSUPPORTED => {
                        bail!("[GL] Surface is incomplete. The combination of internal formats \
                        of the attached images violates an implementation-dependent set of restrictions.");
                    }

                    _ => {
                        bail!("[GL] Surface is incomplete.");
                    }
                }
            }

            data.id = Some(id);
            data.dims = dimensions;
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

        if let Some(v) = surface.id {
            self.ctx.delete_framebuffer(Some(&v));
            check(&self.ctx)?;
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
        let vs = Self::compile(&self.ctx, WebGL::VERTEX_SHADER, vs)?;
        let fs = Self::compile(&self.ctx, WebGL::FRAGMENT_SHADER, fs)?;
        let id = Self::link(&self.ctx, &[vs, fs])?;

        let shader = GLShaderData {
            handle: handle,
            id: id,
            params: params,
            uniforms: RefCell::new(FastHashMap::default()),
            attributes: RefCell::new(FastHashMap::default()),
        };

        for (name, _, _) in shader.params.attributes.iter() {
            let name: &'static str = name.into();
            if let Err(err) = shader.attribute_location(&self.ctx, name) {
                self.ctx.delete_program(Some(&shader.id));
                bail!(err);
            }
        }

        for &(ref name, _) in shader.params.uniforms.iter() {
            if let Err(err) = shader.uniform_location(&self.ctx, name) {
                self.ctx.delete_program(Some(&shader.id));
                bail!(err);
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
        {
            let ctx = &self.ctx;
            self.state.vaos.retain(|&(h, _), vao| {
                if h == shader.handle {
                    ctx.delete_vertex_array(Some(&vao));
                    false
                } else {
                    true
                }
            });
        }

        if self.state.binded_shader == Some(handle) {
            self.state.binded_shader = None;
        }

        self.ctx.delete_program(Some(&shader.id));
        check(&self.ctx)
    }

    unsafe fn create_texture(
        &mut self,
        handle: TextureHandle,
        params: TextureParams,
        data: Option<TextureData>,
    ) -> Result<()> {
        if !self.capabilities.support_texture_format(params.format) {
            bail!(
                "The GL Context does not support the texture format {:?}.",
                params.format
            );
        }

        let id = self.ctx.create_texture().unwrap();
        let mut allocated = false;

        if let Some(mut data) = data {
            let len = data.bytes.len();
            if len > 0 {
                Self::bind_texture(
                    &self.ctx,
                    &mut self.state,
                    Some(Sampler::Texture(handle)),
                    0,
                    Some(&id),
                )?;

                Self::bind_texture_params(&self.ctx, params.wrap, params.filter, len as u32)?;

                let (internal_format, format, pixel_type) = params.format.into();
                let mut dims = (params.dimensions.x as i32, params.dimensions.y as i32);

                if params.format.compressed() {
                    for (i, v) in data.bytes.drain(..).enumerate() {
                        let mv = ::std::slice::from_raw_parts_mut(v.as_ptr() as *mut u8, v.len());
                        self.ctx.compressed_tex_image_2d_with_u8_array(
                            WebGL::TEXTURE_2D,
                            i as i32,
                            internal_format,
                            dims.0,
                            dims.1,
                            0,
                            mv,
                        );

                        dims.0 = (dims.0 / 2).max(1);
                        dims.1 = (dims.1 / 2).max(1);
                    }
                } else {
                    for (i, v) in data.bytes.drain(..).enumerate() {
                        let mv = ::std::slice::from_raw_parts_mut(v.as_ptr() as *mut u8, v.len());
                        self.ctx
                            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                                WebGL::TEXTURE_2D,
                                i as i32,
                                internal_format as i32,
                                dims.0,
                                dims.1,
                                0,
                                format,
                                pixel_type,
                                Some(mv),
                            ).unwrap();

                        dims.0 = (dims.0 / 2).max(1);
                        dims.1 = (dims.1 / 2).max(1);
                    }
                }

                allocated = true;
            }
        }

        check(&self.ctx)?;

        self.textures.create(
            handle,
            GLTextureData {
                handle: handle,
                id: id,
                params: params,
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

        let (internal_format, format, pixel_type) = texture.params.format.into();

        Self::bind_texture(
            &self.ctx,
            &mut self.state,
            Some(Sampler::Texture(handle)),
            0,
            Some(&texture.id),
        )?;

        if !*texture.allocated.borrow() {
            Self::bind_texture_params(&self.ctx, texture.params.wrap, texture.params.filter, 1)?;

            self.ctx
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    WebGL::TEXTURE_2D,
                    0,
                    internal_format as i32,
                    texture.params.dimensions.x as i32,
                    texture.params.dimensions.y as i32,
                    0,
                    format,
                    pixel_type,
                    None,
                ).unwrap();

            *texture.allocated.borrow_mut() = true;
        }

        let mv = ::std::slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len());
        self.ctx
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                WebGL::TEXTURE_2D,
                0,
                area.min.x as i32,
                area.min.y as i32,
                area.dim().x as i32,
                area.dim().y as i32,
                format,
                pixel_type,
                Some(mv),
            ).unwrap();

        check(&self.ctx)
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

        self.ctx.delete_texture(Some(&texture.id));
        check(&self.ctx)
    }

    unsafe fn create_render_texture(
        &mut self,
        handle: RenderTextureHandle,
        params: RenderTextureParams,
    ) -> Result<()> {
        let id = if params.sampler {
            let id = self.ctx.create_texture().unwrap();

            Self::bind_texture(
                &self.ctx,
                &mut self.state,
                Some(Sampler::RenderTexture(handle)),
                0,
                Some(&id),
            )?;
            Self::bind_texture_params(&self.ctx, params.wrap, params.filter, 1)?;

            let (internal_format, format, pixel_type) = params.format.into();
            self.ctx
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    WebGL::TEXTURE_2D,
                    0,
                    internal_format as i32,
                    params.dimensions.x as i32,
                    params.dimensions.y as i32,
                    0,
                    format,
                    pixel_type,
                    None,
                ).unwrap();

            GLRenderTexture::T(id)
        } else {
            let id = self.ctx.create_renderbuffer().unwrap();
            self.ctx.bind_renderbuffer(WebGL::RENDERBUFFER, Some(&id));

            let (internal_format, _, _) = params.format.into();
            self.ctx.renderbuffer_storage(
                WebGL::RENDERBUFFER,
                internal_format,
                params.dimensions.x as i32,
                params.dimensions.y as i32,
            );

            GLRenderTexture::R(id)
        };

        check(&self.ctx)?;

        self.render_textures.create(
            handle,
            GLRenderTextureData {
                handle: handle,
                id: id,
                params: params,
            },
        );

        Ok(())
    }

    unsafe fn delete_render_texture(&mut self, handle: RenderTextureHandle) -> Result<()> {
        let rt = self
            .render_textures
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        match rt.id {
            GLRenderTexture::T(v) => {
                for v in self.state.binded_textures.iter_mut() {
                    if *v == Some(Sampler::RenderTexture(handle)) {
                        *v = None;
                    }
                }

                self.ctx.delete_texture(Some(&v));
            }
            GLRenderTexture::R(v) => {
                self.ctx.delete_renderbuffer(Some(&v));
            }
        }

        check(&self.ctx)
    }

    unsafe fn create_mesh(
        &mut self,
        handle: MeshHandle,
        params: MeshParams,
        data: Option<MeshData>,
    ) -> Result<()> {
        let vbo = Self::create_buffer(
            &self.ctx,
            WebGL::ARRAY_BUFFER,
            params.hint,
            params.vertex_buffer_len(),
            data.as_ref().map(|v| v.vptr.as_ref()),
        )?;

        let ibo = Self::create_buffer(
            &self.ctx,
            WebGL::ELEMENT_ARRAY_BUFFER,
            params.hint,
            params.index_buffer_len(),
            data.as_ref().map(|v| v.iptr.as_ref()),
        )?;

        self.meshes.create(
            handle,
            GLMeshData {
                handle: handle,
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
        let mesh = self
            .meshes
            .get(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if mesh.params.hint == MeshHint::Immutable {
            bail!("Trying to update immutable buffer");
        }

        Self::update_buffer(&self.ctx, WebGL::ARRAY_BUFFER, &mesh.vbo, offset, data)
    }

    unsafe fn update_index_buffer(
        &mut self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        let mesh = self
            .meshes
            .get(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        if mesh.params.hint == MeshHint::Immutable {
            bail!("Trying to update immutable buffer");
        }

        Self::update_buffer(
            &self.ctx,
            WebGL::ELEMENT_ARRAY_BUFFER,
            &mesh.ibo,
            offset,
            data,
        )
    }

    unsafe fn delete_mesh(&mut self, handle: MeshHandle) -> Result<()> {
        let mesh = self
            .meshes
            .free(handle)
            .ok_or_else(|| format_err!("{:?} is invalid.", handle))?;

        // Removes deprecated `VertexArrayObject`s.
        {
            let ctx = &self.ctx;
            self.state.vaos.retain(|&(_, h), vao| {
                if h == mesh.handle {
                    ctx.delete_vertex_array(Some(&vao));
                    false
                } else {
                    true
                }
            });
        }

        self.ctx.delete_buffer(Some(&mesh.vbo));
        self.ctx.delete_buffer(Some(&mesh.ibo));
        check(&self.ctx)
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
        let id = surface.id.as_ref();
        let dimensions = surface.dims.unwrap_or(dimensions);
        self.ctx.bind_framebuffer(WebGL::FRAMEBUFFER, id);

        // Reset the viewport and scissor box.
        let vp = SurfaceViewport {
            position: Vector2::new(0, 0),
            size: dimensions,
        };

        Self::set_viewport(&self.ctx, &mut self.state, vp)?;
        Self::set_scissor(&self.ctx, &mut self.state, SurfaceScissor::Disable)?;

        if !self.state.cleared_surfaces.contains(&handle) {
            // Sets depth write enable to make sure that we can clear depth buffer properly.
            if surface.params.clear_depth.is_some() {
                self.state.binded_shader = None;
                Self::set_depth_test(&self.ctx, &mut self.state, true, Comparison::Always)?;
            }

            // Clears frame buffer.
            Self::clear(
                &self.ctx,
                surface.params.clear_color,
                surface.params.clear_depth,
                surface.params.clear_stencil,
            )?;

            self.state.cleared_surfaces.insert(handle);
        }

        self.state.binded_surface = Some(handle);
        Ok(())
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

        Self::bind_shader(&self.ctx, &mut self.state, &shader)?;

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
                        Self::bind_uniform_variable(&self.ctx, &location, &v)?;

                        if let Some(texture) = self.textures.get(handle) {
                            Self::bind_texture(
                                &self.ctx,
                                &mut self.state,
                                Some(Sampler::Texture(handle)),
                                index,
                                Some(&texture.id),
                            )?;
                        } else {
                            Self::bind_texture(&self.ctx, &mut self.state, None, index, None)?;
                        }

                        index += 1;
                    }
                    UniformVariable::RenderTexture(handle) => {
                        let v = UniformVariable::I32(index as i32);
                        Self::bind_uniform_variable(&self.ctx, &location, &v)?;

                        if let Some(texture) = self.render_textures.get(handle) {
                            match texture.id {
                                GLRenderTexture::T(ref w) => {
                                    Self::bind_texture(
                                        &self.ctx,
                                        &mut self.state,
                                        Some(Sampler::RenderTexture(handle)),
                                        index,
                                        Some(w),
                                    )?;
                                }
                                _ => {
                                    bail!("The render buffer does not have a sampler.");
                                }
                            }
                        } else {
                            Self::bind_texture(&self.ctx, &mut self.state, None, index, None)?;
                        }

                        index += 1;
                    }
                    _ => {
                        Self::bind_uniform_variable(&self.ctx, &location, &variable)?;
                    }
                }
            } else {
                bail!("Undefined uniform field {:?}.", field);
            }
        }

        if let Some(mesh) = self.meshes.get(mesh) {
            // Bind vertex buffer and vertex array object.
            Self::bind_mesh(&self.ctx, &mut self.state, &shader, &mesh)?;

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

            self.ctx.draw_elements_with_i32(
                mesh.params.primitive.into(),
                len as i32,
                mesh.params.index_format.into(),
                from as i32,
            );

            check(&self.ctx)?;
            Ok(mesh.params.primitive.assemble(len as u32))
        } else {
            Ok(0)
        }
    }

    unsafe fn update_surface_scissor(&mut self, scissor: SurfaceScissor) -> Result<()> {
        Self::set_scissor(&self.ctx, &mut self.state, scissor)
    }

    unsafe fn update_surface_viewport(&mut self, vp: SurfaceViewport) -> Result<()> {
        Self::set_viewport(&self.ctx, &mut self.state, vp)
    }

    unsafe fn flush(&mut self) -> Result<()> {
        self.ctx.finish();
        Ok(())
    }
}

impl WebGLVisitor {
    unsafe fn bind_surface_render_texture(
        ctx: &WebGL,
        rt: &GLRenderTextureData,
        index: usize,
    ) -> Result<()> {
        let location = match rt.params.format {
            RenderTextureFormat::RGB8 | RenderTextureFormat::RGBA4 | RenderTextureFormat::RGBA8 => {
                WebGL::COLOR_ATTACHMENT0 + index as u32
            }
            RenderTextureFormat::Depth16
            | RenderTextureFormat::Depth24
            | RenderTextureFormat::Depth32 => WebGL::DEPTH_ATTACHMENT,
            RenderTextureFormat::Depth24Stencil8 => WebGL::DEPTH_STENCIL_ATTACHMENT,
        };

        match rt.id {
            GLRenderTexture::T(ref v) => ctx.framebuffer_texture_2d(
                WebGL::FRAMEBUFFER,
                location,
                WebGL::TEXTURE_2D,
                Some(v),
                0,
            ),
            GLRenderTexture::R(ref v) => ctx.framebuffer_renderbuffer(
                WebGL::FRAMEBUFFER,
                location,
                WebGL::RENDERBUFFER,
                Some(v),
            ),
        }

        check(&ctx)
    }
}

impl WebGLVisitor {
    unsafe fn compile(ctx: &WebGL, tp: u32, src: &str) -> Result<WebGlShader> {
        let shader = ctx
            .create_shader(tp)
            .ok_or_else(|| "Unable to create shader object".into())
            .map_err(|err: String| format_err!("{}", err))?;

        ctx.shader_source(&shader, src);
        ctx.compile_shader(&shader);

        if ctx
            .get_shader_parameter(&shader, WebGL::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            let err = ctx
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into());

            bail!(err);
        }
    }

    unsafe fn link<'a, T>(ctx: &WebGL, shaders: T) -> Result<WebGlProgram>
    where
        T: IntoIterator<Item = &'a WebGlShader>,
    {
        let program = ctx
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))
            .map_err(|err| format_err!("{}", err))?;

        for shader in shaders {
            ctx.attach_shader(&program, shader)
        }
        ctx.link_program(&program);

        if ctx
            .get_program_parameter(&program, WebGL::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            let err = ctx
                .get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown error creating program object".into());

            bail!(err);
        }
    }

    unsafe fn bind_shader(
        ctx: &WebGL,
        state: &mut WebGLState,
        shader: &GLShaderData,
    ) -> Result<()> {
        if state.binded_shader == Some(shader.handle) {
            return Ok(());
        }

        ctx.use_program(Some(&shader.id));
        check(ctx)?;

        let rs = &shader.params.state;
        Self::set_cull_face(ctx, state, rs.cull_face)?;
        Self::set_front_face_order(ctx, state, rs.front_face_order)?;
        Self::set_depth_test(ctx, state, rs.depth_write, rs.depth_test)?;
        Self::set_depth_write_offset(ctx, state, rs.depth_write_offset)?;
        Self::set_color_blend(ctx, state, rs.color_blend)?;
        Self::set_color_write(ctx, state, rs.color_write)?;

        state.binded_shader = Some(shader.handle);
        Ok(())
    }

    unsafe fn bind_mesh(
        ctx: &WebGL,
        state: &mut WebGLState,
        shader: &GLShaderData,
        mesh: &GLMeshData,
    ) -> Result<()> {
        assert!(state.binded_shader == Some(shader.handle));

        let k = (shader.handle, mesh.handle);
        if state.binded_vao != Some(k) {
            if let Some(vao) = state.vaos.get(&k).cloned() {
                ctx.bind_vertex_array(Some(&vao));
                check(ctx)?;
            } else {
                let vao = ctx.create_vertex_array().unwrap();
                ctx.bind_vertex_array(Some(&vao));
                ctx.bind_buffer(WebGL::ARRAY_BUFFER, Some(&mesh.vbo));

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

                        let location = shader.attribute_location(ctx, name.into())?;
                        ctx.enable_vertex_attrib_array(location as u32);
                        ctx.vertex_attrib_pointer_with_i32(
                            location as u32,
                            element.size as i32,
                            element.format.into(),
                            element.normalized,
                            stride as i32,
                            offset as i32,
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

                check(ctx)?;
                state.vaos.insert(k, vao);
            }

            state.binded_vao = Some(k);
        }

        ctx.bind_buffer(WebGL::ELEMENT_ARRAY_BUFFER, Some(&mesh.ibo));
        Ok(())
    }

    unsafe fn bind_uniform_variable(
        ctx: &WebGL,
        location: &WebGlUniformLocation,
        variable: &UniformVariable,
    ) -> Result<()> {
        match *variable {
            UniformVariable::Texture(_) => unreachable!(),
            UniformVariable::RenderTexture(_) => unreachable!(),
            UniformVariable::I32(v) => ctx.uniform1i(Some(&location), v),
            UniformVariable::F32(v) => ctx.uniform1f(Some(&location), v),
            UniformVariable::Vector2f(v) => ctx.uniform2f(Some(&location), v[0], v[1]),
            UniformVariable::Vector3f(v) => ctx.uniform3f(Some(&location), v[0], v[1], v[2]),
            UniformVariable::Vector4f(v) => ctx.uniform4f(Some(&location), v[0], v[1], v[2], v[3]),
            UniformVariable::Matrix2f(v, transpose) => {
                let mv = ::std::slice::from_raw_parts_mut(v.as_ptr() as *mut f32, 4);
                ctx.uniform_matrix2fv_with_f32_array(Some(&location), transpose, mv)
            }
            UniformVariable::Matrix3f(v, transpose) => {
                let mv = ::std::slice::from_raw_parts_mut(v.as_ptr() as *mut f32, 9);
                ctx.uniform_matrix3fv_with_f32_array(Some(&location), transpose, mv)
            }
            UniformVariable::Matrix4f(v, transpose) => {
                let mv = ::std::slice::from_raw_parts_mut(v.as_ptr() as *mut f32, 16);
                ctx.uniform_matrix4fv_with_f32_array(Some(&location), transpose, mv)
            }
        }

        check(ctx)
    }

    unsafe fn reset_render_state(ctx: &WebGL, state: &mut WebGLState) -> Result<()> {
        let rs = &mut state.render_state;

        ctx.disable(WebGL::CULL_FACE);
        rs.cull_face = CullFace::Nothing;

        ctx.front_face(WebGL::CCW);
        rs.front_face_order = FrontFaceOrder::CounterClockwise;

        ctx.disable(WebGL::DEPTH_TEST);
        ctx.depth_mask(false);
        rs.depth_write = false;
        ctx.depth_func(WebGL::ALWAYS);
        rs.depth_test = Comparison::Always;
        ctx.disable(WebGL::POLYGON_OFFSET_FILL);
        rs.depth_write_offset = None;

        ctx.disable(WebGL::BLEND);
        rs.color_blend = None;

        ctx.color_mask(true, true, true, true);
        rs.color_write = (true, true, true, true);

        ctx.disable(WebGL::SCISSOR_TEST);
        state.scissor = SurfaceScissor::Disable;

        ctx.pixel_storei(WebGL::UNPACK_ALIGNMENT, 1);
        ctx.bind_framebuffer(WebGL::FRAMEBUFFER, None);

        check(&ctx)
    }

    /// Specify whether front- or back-facing polygons can be culled.
    unsafe fn set_cull_face(ctx: &WebGL, state: &mut WebGLState, face: CullFace) -> Result<()> {
        if state.render_state.cull_face != face {
            if face != CullFace::Nothing {
                ctx.enable(WebGL::CULL_FACE);
                ctx.cull_face(match face {
                    CullFace::Front => WebGL::FRONT,
                    CullFace::Back => WebGL::BACK,
                    _ => unreachable!(""),
                });
            } else {
                ctx.disable(WebGL::CULL_FACE);
            }

            state.render_state.cull_face = face;
            check(&ctx)?;
        }

        Ok(())
    }

    /// Define front- and back-facing polygons.
    unsafe fn set_front_face_order(
        ctx: &WebGL,
        state: &mut WebGLState,
        front: FrontFaceOrder,
    ) -> Result<()> {
        if state.render_state.front_face_order != front {
            ctx.front_face(match front {
                FrontFaceOrder::Clockwise => WebGL::CW,
                FrontFaceOrder::CounterClockwise => WebGL::CCW,
            });

            state.render_state.front_face_order = front;
            check(&ctx)?;
        }

        Ok(())
    }

    /// Enable or disable writing into the depth buffer and specify the value used for depth
    /// buffer comparisons.
    unsafe fn set_depth_test(
        ctx: &WebGL,
        state: &mut WebGLState,
        write: bool,
        comparsion: Comparison,
    ) -> Result<()> {
        let state = &mut state.render_state;

        // Note that even if the depth buffer exists and the depth mask is non-zero,
        // the depth buffer is not updated if the depth test is disabled.
        let enable = comparsion != Comparison::Always || write;
        let last_enable = state.depth_test != Comparison::Always || state.depth_write;
        if enable != last_enable {
            if enable {
                ctx.enable(WebGL::DEPTH_TEST);
            } else {
                ctx.disable(WebGL::DEPTH_TEST);
            }
        }

        if state.depth_write != write {
            if write {
                ctx.depth_mask(true);
            } else {
                ctx.depth_mask(false);
            }

            state.depth_write = write;
        }

        if state.depth_test != comparsion {
            ctx.depth_func(comparsion.into());
            state.depth_test = comparsion;
        }

        check(&ctx)
    }

    /// Set `offset` to address the scale and units used to calculate depth values.
    unsafe fn set_depth_write_offset(
        ctx: &WebGL,
        state: &mut WebGLState,
        offset: Option<(f32, f32)>,
    ) -> Result<()> {
        let state = &mut state.render_state;

        if state.depth_write_offset != offset {
            if let Some(v) = offset {
                if v.0 != 0.0 || v.1 != 0.0 {
                    ctx.enable(WebGL::POLYGON_OFFSET_FILL);
                    ctx.polygon_offset(v.0, v.1);
                } else {
                    ctx.disable(WebGL::POLYGON_OFFSET_FILL);
                }
            }

            state.depth_write_offset = offset;
            check(&ctx)?;
        }

        Ok(())
    }

    // Specifies how source and destination are combined.
    unsafe fn set_color_blend(
        ctx: &WebGL,
        state: &mut WebGLState,
        blend: Option<(Equation, BlendFactor, BlendFactor)>,
    ) -> Result<()> {
        let state = &mut state.render_state;

        if state.color_blend != blend {
            if let Some((equation, src, dst)) = blend {
                if state.color_blend == None {
                    ctx.enable(WebGL::BLEND);
                }

                ctx.blend_func(src.into(), dst.into());
                ctx.blend_equation(equation.into());
            } else if state.color_blend != None {
                ctx.disable(WebGL::BLEND);
            }

            state.color_blend = blend;
            check(&ctx)?;
        }

        Ok(())
    }

    /// Enable or disable writing color elements into the color buffer.
    unsafe fn set_color_write(
        ctx: &WebGL,
        state: &mut WebGLState,
        mask: (bool, bool, bool, bool),
    ) -> Result<()> {
        let state = &mut state.render_state;

        if state.color_write != mask {
            ctx.color_mask(mask.0, mask.1, mask.2, mask.3);
            state.color_write = mask;
            check(&ctx)?;
        }

        Ok(())
    }

    /// Set the scissor box relative to the top-lef corner of th window, in pixels.
    unsafe fn set_scissor(
        ctx: &WebGL,
        state: &mut WebGLState,
        scissor: SurfaceScissor,
    ) -> Result<()> {
        match scissor {
            SurfaceScissor::Disable => if state.scissor != SurfaceScissor::Disable {
                ctx.disable(WebGL::SCISSOR_TEST);
            },
            SurfaceScissor::Enable { position, size } => {
                if state.scissor == SurfaceScissor::Disable {
                    ctx.enable(WebGL::SCISSOR_TEST);
                }

                ctx.scissor(
                    position.x as i32,
                    position.y as i32,
                    size.x as i32,
                    size.y as i32,
                );
            }
        }

        state.scissor = scissor;
        check(&ctx)
    }

    /// Set the viewport relative to the top-lef corner of th window, in pixels.
    unsafe fn set_viewport(ctx: &WebGL, state: &mut WebGLState, vp: SurfaceViewport) -> Result<()> {
        if state.view != vp {
            ctx.viewport(
                vp.position.x as i32,
                vp.position.y as i32,
                vp.size.x as i32,
                vp.size.y as i32,
            );

            state.view = vp;
            check(&ctx)?;
        }

        Ok(())
    }

    unsafe fn clear<C, D, S>(ctx: &WebGL, color: C, depth: D, stencil: S) -> Result<()>
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
            bits |= WebGL::COLOR_BUFFER_BIT;
            ctx.clear_color(v.r, v.g, v.b, v.a);
        }

        if let Some(v) = depth {
            bits |= WebGL::DEPTH_BUFFER_BIT;
            ctx.clear_depth(v);
        }

        if let Some(v) = stencil {
            bits |= WebGL::STENCIL_BUFFER_BIT;
            ctx.clear_stencil(v);
        }

        if bits != 0 {
            ctx.clear(bits);
            check(&ctx)
        } else {
            Ok(())
        }
    }
}

impl WebGLVisitor {
    unsafe fn bind_texture(
        ctx: &WebGL,
        state: &mut WebGLState,
        sampler: Option<Sampler>,
        index: usize,
        id: Option<&WebGlTexture>,
    ) -> Result<()> {
        if state.binded_texture_index != index {
            ctx.active_texture(WebGL::TEXTURE0 + index as u32);
            state.binded_texture_index = index;
        }

        if state.binded_textures.len() <= index {
            state.binded_textures.resize(index + 1, None);
        }

        if state.binded_textures[index] != sampler {
            state.binded_textures[index] = sampler;
            ctx.bind_texture(WebGL::TEXTURE_2D, id);
        }

        check(ctx)
    }

    unsafe fn bind_texture_params(
        ctx: &WebGL,
        wrap: TextureWrap,
        filter: TextureFilter,
        levels: u32,
    ) -> Result<()> {
        let wrap: u32 = wrap.into();
        let wrap = wrap as i32;

        ctx.tex_parameteri(WebGL::TEXTURE_2D, WebGL::TEXTURE_WRAP_S, wrap);
        ctx.tex_parameteri(WebGL::TEXTURE_2D, WebGL::TEXTURE_WRAP_T, wrap);

        match filter {
            TextureFilter::Nearest => {
                let min_filter = if levels > 1 {
                    WebGL::NEAREST_MIPMAP_NEAREST
                } else {
                    WebGL::NEAREST
                } as i32;

                ctx.tex_parameteri(WebGL::TEXTURE_2D, WebGL::TEXTURE_MIN_FILTER, min_filter);

                ctx.tex_parameteri(
                    WebGL::TEXTURE_2D,
                    WebGL::TEXTURE_MAG_FILTER,
                    WebGL::NEAREST as i32,
                );
            }
            TextureFilter::Linear => {
                let min_filter = if levels > 1 {
                    WebGL::LINEAR_MIPMAP_LINEAR
                } else {
                    WebGL::LINEAR
                } as i32;

                ctx.tex_parameteri(WebGL::TEXTURE_2D, WebGL::TEXTURE_MIN_FILTER, min_filter);
                ctx.tex_parameteri(
                    WebGL::TEXTURE_2D,
                    WebGL::TEXTURE_MAG_FILTER,
                    WebGL::LINEAR as i32,
                );
            }
        }

        if levels > 1 {
            ctx.tex_parameteri(WebGL::TEXTURE_2D, WebGL::TEXTURE_BASE_LEVEL, 0);
            ctx.tex_parameteri(
                WebGL::TEXTURE_2D,
                WebGL::TEXTURE_MAX_LEVEL,
                (levels - 1) as i32,
            );
        }

        check(&ctx)
    }
}

impl WebGLVisitor {
    unsafe fn create_buffer(
        ctx: &WebGL,
        target: u32,
        hint: MeshHint,
        size: usize,
        data: Option<&[u8]>,
    ) -> Result<WebGlBuffer> {
        let id = ctx.create_buffer().unwrap();
        ctx.bind_buffer(target, Some(&id));
        check(&ctx)?;

        let hint = hint.into();
        match data {
            Some(v) => {
                let mv = ::std::slice::from_raw_parts_mut(v.as_ptr() as *mut u8, v.len());
                ctx.buffer_data_with_u8_array(target, mv, hint)
            }
            _ => ctx.buffer_data_with_i32(target, size as i32, hint),
        }

        check(&ctx)?;
        Ok(id)
    }

    unsafe fn update_buffer(
        ctx: &WebGL,
        target: u32,
        id: &WebGlBuffer,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        let mv = ::std::slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len());
        ctx.bind_buffer(target, Some(&id));
        ctx.buffer_sub_data_with_i32_and_u8_array(target, offset as i32, mv);
        check(&ctx)
    }
}

unsafe fn check(ctx: &WebGL) -> Result<()> {
    match ctx.get_error() {
        WebGL::NO_ERROR => Ok(()),

        WebGL::INVALID_ENUM => {
            bail!("[WebGL] An unacceptable value is specified for an enumerated argument.")
        }

        WebGL::INVALID_VALUE => bail!("[WebGL] A numeric argument is out of range."),

        WebGL::INVALID_OPERATION => {
            bail!("[WebGL] The specified operation is not allowed in the current state.")
        }

        WebGL::INVALID_FRAMEBUFFER_OPERATION => bail!(
            r"[WebGL] The command is trying to render to or read from the framebufferwhile the \
            currently bound framebuffer is not framebuffer complete."
        ),

        WebGL::OUT_OF_MEMORY => {
            bail!("[WebGL] There is not enough memory left to execute the command.")
        }

        _ => bail!("[WebGL] Oops, Unknown OpenGL error."),
    }
}
