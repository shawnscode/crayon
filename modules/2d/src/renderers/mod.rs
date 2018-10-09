use std::sync::Arc;

use crayon::errors::*;
use crayon::video::prelude::*;

pub const MAX_VERTICES: usize = 4096;

impl_vertex! {
    Vertex {
        position => [Position; Float; 2; false],
        uv => [Texcoord0; Float; 2; false],
        diffuse => [Color0; UByte; 4; true],
        additive => [Color1; UByte; 4; true],
    }
}

pub struct Renderer {
    video: Arc<VideoSystemShared>,

    surface: SurfaceHandle,
    shader: ShaderHandle,
    mesh: MeshHandle,
    white_texture: TextureHandle,

    active_texture: Option<TextureHandle>,
    buf: Vec<Vertex>,
    batch: Batch,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.video.delete_surface(self.surface);
        self.video.delete_shader(self.shader);
        self.video.delete_mesh(self.mesh);
    }
}

impl Renderer {
    /// Creates a new `Renderer`. This will allocates essential video resources in background.
    pub fn new(video: Arc<VideoSystemShared>) -> Result<Self> {
        let mut params = SurfaceParams::default();
        params.set_clear(None, None, None);
        let surface = video.create_surface(params)?;

        let layout = AttributeLayout::build()
            .with(Attribute::Position, 2)
            .with(Attribute::Texcoord0, 2)
            .with(Attribute::Color0, 4)
            .with(Attribute::Color1, 4)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("u_MainTex", UniformVariableType::Matrix4f)
            .finish();

        let mut render_state = RenderState::default();
        render_state.cull_face = CullFace::Back;
        render_state.color_blend = Some((
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        ));

        let mut params = ShaderParams::default();
        params.attributes = layout;
        params.uniforms = uniforms;
        params.state = render_state;

        let vs = include_str!("shaders/texture.vs").to_owned();
        let fs = include_str!("shaders/texture.fs").to_owned();
        let shader = video.create_shader(params, vs, fs)?;

        let mut params = TextureParams::default();
        params.dimensions = (2, 2).into();

        let bytes = vec![255; 16];
        let data = TextureData {
            bytes: vec![bytes.into_boxed_slice()],
        };

        let white_texture = video.create_texture(params, data)?;

        let mut params = MeshParams::default();
        params.hint = MeshHint::Stream;
        params.layout = Vertex::layout();
        params.index_format = IndexFormat::U16;
        params.primitive = MeshPrimitive::Triangles;
        params.num_verts = MAX_VERTICES;
        params.num_idxes = MAX_VERTICES;

        let mesh = video.create_mesh(params, None)?;

        let mut idxes: Vec<u16> = Vec::with_capacity(MAX_VERTICES);
        idxes.resize(MAX_VERTICES, 0);

        for (i, v) in idxes.iter_mut().enumerate() {
            *v = i as u16;
        }

        video.update_index_buffer(mesh, 0, IndexFormat::encode(&idxes))?;

        Ok(Renderer {
            video: video,

            surface: surface,
            shader: shader,
            mesh: mesh,
            white_texture: white_texture,

            active_texture: None,
            buf: Vec::new(),
            batch: Batch::new(),
        })
    }

    pub fn draw<T: Into<Option<TextureHandle>>>(&mut self, texture: T, buf: &[Vertex]) {
        let texture = texture.into();
        if texture != self.active_texture {
            self.flush();
            self.active_texture = texture;
        }

        let mut iter = 0;
        let mut len = (MAX_VERTICES - self.buf.len()).min(buf.len());
        while (self.buf.len() + len) >= MAX_VERTICES {
            self.buf.extend(&buf[iter..len]);
            self.flush();

            iter += len;
            len = (MAX_VERTICES - self.buf.len()).min(buf.len() - iter);
        }

        self.buf.extend(&buf[iter..]);
    }

    pub fn flush(&mut self) {
        self.batch
            .update_vertex_buffer(self.mesh, 0, Vertex::encode(&self.buf));

        let mut dc = DrawCall::new(self.shader, self.mesh);

        dc.set_uniform_variable(
            "u_MainTex",
            self.active_texture.unwrap_or(self.white_texture),
        );

        dc.mesh_index = MeshIndex::Ptr(0, self.buf.len());
        self.batch.draw(dc);
        self.buf.clear();
    }
}
