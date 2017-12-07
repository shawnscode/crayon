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
    video: Arc<graphics::GraphicsSystemShared>,

    vso: graphics::ViewStateHandle,
    pso: graphics::PipelineStateHandle,
    vbo: graphics::VertexBufferHandle,
    ibo: graphics::IndexBufferHandle,
    label: graphics::utils::GraphicsResourceLabel,

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
        let mut label = graphics::utils::GraphicsResourceLabel::new();

        let mut setup = graphics::ViewStateSetup::default();
        setup.sequence = true;
        setup.clear_color = Some(utils::Color::gray());
        let vso = label.push(video.create_view(setup)?);

        let layout = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .with(graphics::VertexAttribute::Texcoord0, 2)
            .with(graphics::VertexAttribute::Color0, 4)
            .finish();

        let mut setup = graphics::PipelineStateSetup::default();
        setup.layout = layout;
        setup.state.color_blend =
            Some((graphics::Equation::Add,
                  graphics::BlendFactor::Value(graphics::BlendValue::SourceAlpha),
                  graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha)));

        let vs = include_str!("../resources/canvas.vs").to_owned();
        let fs = include_str!("../resources/canvas.fs").to_owned();
        let pso = label.push(video.create_pipeline(setup, vs, fs)?);

        let mut setup = graphics::VertexBufferSetup::default();
        setup.layout = CanvasVertex::layout();
        setup.num = MAX_VERTICES;
        setup.hint = graphics::BufferHint::Stream;

        let vbo = label.push(video.create_vertex_buffer(setup, None)?);

        let mut setup = graphics::IndexBufferSetup::default();
        setup.format = graphics::IndexFormat::U16;
        setup.num = MAX_VERTICES * 2;
        setup.hint = graphics::BufferHint::Stream;

        let ibo = label.push(video.create_index_buffer(setup, None)?);

        Ok(CanvasRenderer {
               video: video.clone(),

               vso: vso,
               pso: pso,
               vbo: vbo,
               ibo: ibo,
               label: label,

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
                  texture: graphics::TextureHandle)
                  -> Result<()> {
        assert!(verts.len() <= MAX_VERTICES);

        if (self.verts.len() + verts.len()) >= MAX_VERTICES ||
           (self.current_texture.is_some() && self.current_texture != Some(texture)) {
            self.flush()?;
        }

        if idxes.len() <= 0 {
            return Ok(());
        }

        let offset = self.verts.len() as u16;
        self.current_texture = Some(texture);

        for &v in verts {
            let mut v = v;
            v.position = self.transform(v.position);
            self.verts.push(v);
        }

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
            self.video.update_vertex_buffer(self.vbo, 0, slice)?;

            let slice = graphics::IndexFormat::as_bytes(&self.idxes);
            self.video.update_index_buffer(self.ibo, 0, slice)?;
        }

        let mut dc = self.video.make();

        if let Some(handle) = self.current_texture {
            dc.with_texture("mainTexture", handle);
        }

        dc.with_view(self.vso)
            .with_pipeline(self.pso)
            .with_data(self.vbo, Some(self.ibo))
            .submit(graphics::Primitive::Triangles, 0, self.idxes.len() as u32)?;

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

impl Drop for CanvasRenderer {
    fn drop(&mut self) {
        self.label.clear(&self.video);
    }
}