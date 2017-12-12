use std::sync::Arc;
use crayon::{graphics, application, math, utils};
use crayon::math::One;

use errors::*;

impl_vertex!{
    CanvasVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        color => [Color0; UByte; 4; true],
    }
}

const MAX_VERTICES: usize = ::std::u16::MAX as usize;

/// The renderer of canvas system.
pub struct CanvasRenderer {
    _video_label: graphics::RAIIGuard,
    video: Arc<graphics::GraphicsSystemShared>,

    surface: graphics::SurfaceHandle,
    shader: graphics::ShaderHandle,
    vbo: graphics::VertexBufferHandle,
    ibo: graphics::IndexBufferHandle,

    verts: Vec<CanvasVertex>,
    idxes: Vec<u16>,

    current_matrix: math::Matrix4<f32>,
    current_texture: Option<graphics::TextureHandle>,
}

impl CanvasRenderer {
    /// Creates a new `CanvasRenderer`. This will allocates essential video
    /// resources in background.
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>();
        let mut label = graphics::RAIIGuard::new(video.clone());

        let mut setup = graphics::SurfaceSetup::default();
        setup.clear_color = Some(utils::Color::gray());
        let surface = label.create_view(setup)?;

        let layout = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .with(graphics::VertexAttribute::Texcoord0, 2)
            .with(graphics::VertexAttribute::Color0, 4)
            .finish();

        let mut setup = graphics::ShaderSetup::default();
        setup.layout = layout;
        setup.render_state.color_blend =
            Some((graphics::Equation::Add,
                  graphics::BlendFactor::Value(graphics::BlendValue::SourceAlpha),
                  graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha)));

        setup.vs = include_str!("../resources/canvas.vs").to_owned();
        setup.fs = include_str!("../resources/canvas.fs").to_owned();

        setup.uniform_variables.push("mainTexture".into());
        let shader = label.create_shader(setup)?;

        let mut setup = graphics::VertexBufferSetup::default();
        setup.layout = CanvasVertex::layout();
        setup.num = MAX_VERTICES;
        setup.hint = graphics::BufferHint::Stream;

        let vbo = label.create_vertex_buffer(setup, None)?;

        let mut setup = graphics::IndexBufferSetup::default();
        setup.format = graphics::IndexFormat::U16;
        setup.num = MAX_VERTICES * 2;
        setup.hint = graphics::BufferHint::Stream;

        let ibo = label.create_index_buffer(setup, None)?;

        Ok(CanvasRenderer {
               _video_label: label,
               video: video.clone(),

               surface: surface,
               shader: shader,
               vbo: vbo,
               ibo: ibo,

               verts: Vec::new(),
               idxes: Vec::new(),

               current_texture: None,
               current_matrix: math::Matrix4::one(),
           })
    }

    /// Set the current transformation matrix, it will be applied to all the
    /// vertices your submitted later.
    pub fn set_matrix(&mut self, matrix: math::Matrix4<f32>) {
        self.current_matrix = matrix;
    }

    /// Submits data for drawing. Notes that the renderer will trying to batch
    /// vertices and indices your submitted to avoid unnessessary draw call.
    pub fn submit(&mut self,
                  verts: &[CanvasVertex],
                  idxes: &[u16],
                  texture: Option<graphics::TextureHandle>)
                  -> Result<()> {
        assert!(verts.len() <= MAX_VERTICES);

        if (self.verts.len() + verts.len()) >= MAX_VERTICES || self.current_texture != texture {
            self.flush()?;
        }

        if idxes.len() <= 0 {
            return Ok(());
        }

        self.current_texture = texture;

        for &v in verts {
            let mut v = v;
            v.position = self.transform(v.position);
            self.verts.push(v);
        }

        let offset = self.verts.len() as u16;
        for &i in idxes {
            assert!(i < verts.len() as u16,
                    "Invalid index into vertices you submitted.");
            self.idxes.push(i + offset);
        }

        Ok(())
    }

    /// Flush the batched data into video card.
    pub fn flush(&mut self) -> Result<()> {
        if self.idxes.len() <= 0 {
            return Ok(());
        }

        {
            let slice = CanvasVertex::as_bytes(&self.verts);
            let task = graphics::BucketTask::update_vertex_buffer(self.vbo, 0, slice);
            self.video.submit(self.surface, task)?;

            let slice = graphics::IndexFormat::as_bytes(&self.idxes);
            let task = graphics::BucketTask::update_index_buffer(self.ibo, 0, slice);
            self.video.submit(self.surface, task)?;
        }

        {
            let mut dc = graphics::DrawCall::new(self.shader);

            if let Some(texture) = self.current_texture {
                dc.set_uniform_variable("mainTexture", texture);
            }

            dc.set_mesh(self.vbo, self.ibo);

            let task = dc.draw(graphics::Primitive::Triangles, 0, self.idxes.len() as u32)?;
            self.video.submit(self.surface, task)?;
        }

        self.verts.clear();
        self.idxes.clear();
        Ok(())
    }

    #[inline(always)]
    fn transform(&self, position: [f32; 2]) -> [f32; 2] {
        let p = math::Vector4::new(position[0], position[1], 0.0, 1.0);
        let p = self.current_matrix * p;
        [p.x, p.y]
    }
}