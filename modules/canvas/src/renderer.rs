use std::sync::Arc;
use crayon::{graphics, application};

use errors::*;

impl_vertex!{
    CanvasVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        color => [Color0; UByte; 4; true],
    }
}

pub struct CanvasRenderer {
    video: Arc<graphics::GraphicsSystemShared>,

    vso: graphics::ViewStateHandle,
    pso: graphics::PipelineStateHandle,

    vbo_bufs: Vec<graphics::VertexBufferHandle>,
    ibo_bufs: Vec<graphics::IndexBufferHandle>,

    verts: Vec<CanvasVertex>,
    verts_strips: Vec<usize>,
    verts_chunk_num: usize,

    idxes: Vec<u16>,
    idxes_strips: Vec<usize>,

    texture: Option<graphics::TextureHandle>,
}

impl CanvasRenderer {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>();

        let mut setup = graphics::ViewStateSetup::default();
        setup.sequence = true;
        setup.clear_color = Some(graphics::Color::gray());
        let vso = video.create_view(setup)?;

        let layout = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .with(graphics::VertexAttribute::Texcoord0, 2)
            .with(graphics::VertexAttribute::Color0, 4)
            .finish();

        let mut setup = graphics::PipelineStateSetup::default();
        setup.layout = layout;
        setup.state.color_blend =
            Some((graphics::Equation::Add,
                  graphics::BlendFactor::One,
                  graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha)));

        let vs = include_str!("../assets/canvas.vs").to_owned();
        let fs = include_str!("../assets/canvas.fs").to_owned();
        let pso = video.create_pipeline(setup, vs, fs)?;

        Ok(CanvasRenderer {
               video: video.clone(),

               vso: vso,
               pso: pso,

               vbo_bufs: Vec::new(),
               ibo_bufs: Vec::new(),

               verts: Vec::new(),
               verts_strips: Vec::new(),
               verts_chunk_num: 4096,

               idxes: Vec::new(),
               idxes_strips: Vec::new(),

               texture: None,
           })
    }

    pub fn clear(&mut self) {
        for handle in self.vbo_bufs.drain(..) {
            self.video.delete_vertex_buffer(handle);
        }

        for handle in self.ibo_bufs.drain(..) {
            self.video.delete_index_buffer(handle);
        }
    }

    pub fn draw(&mut self,
                verts: &[CanvasVertex],
                idxes: &[u16],
                texture: graphics::TextureHandle) {
        self.fill_vertices(&verts, &idxes);

        if self.texture.is_some() && Some(texture) != self.texture {
            self.texture = Some(texture);
            self.flush();
        }
    }

    pub fn fill_vertices(&mut self, verts: &[CanvasVertex], idxes: &[u16]) {
        assert!(verts.len() <= self.verts_chunk_num);
        assert!(idxes.len() <= self.verts_chunk_num * 2);

        let start = if self.verts_strips.len() == 0 {
            0
        } else {
            self.verts_strips[self.verts_strips.len() - 1]
        };

        let mut offset = self.verts.len() - start;
        if (self.verts_chunk_num - offset) < verts.len() {
            self.verts_strips.push(self.verts.len());
            self.idxes_strips.push(self.idxes.len());
            offset = 0;
        }

        self.verts.extend_from_slice(verts);
        for i in idxes {
            self.idxes.push(*i + offset as u16);
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        assert!(self.idxes_strips.len() == self.verts_strips.len());

        if self.idxes.len() <= 0 {
            return Ok(());
        }

        if (self.idxes_strips.len() == 0) ||
           (self.idxes.len() > self.idxes_strips[self.idxes_strips.len() - 1]) {
            self.idxes_strips.push(self.idxes.len());
            self.verts_strips.push(self.verts.len());
        }

        for _ in self.vbo_bufs.len()..self.verts_strips.len() {
            let mut setup = graphics::VertexBufferSetup::default();
            setup.layout = CanvasVertex::layout();
            setup.num = self.verts_chunk_num;
            setup.hint = graphics::BufferHint::Stream;

            let handle = self.video.create_vertex_buffer(setup, None)?;
            self.vbo_bufs.push(handle);
        }

        for i in 0..self.verts_strips.len() {
            let start = if i > 0 { self.verts_strips[i - 1] } else { 0 };
            let end = self.verts_strips[i];
            assert!((end - start) < self.verts_chunk_num);

            let slice = CanvasVertex::as_bytes(&self.verts[start..end]);
            self.video.update_vertex_buffer(self.vbo_bufs[i], 0, slice)?;
        }

        for _ in self.ibo_bufs.len()..self.idxes_strips.len() {
            let mut setup = graphics::IndexBufferSetup::default();
            setup.format = graphics::IndexFormat::U16;
            setup.num = self.verts_chunk_num * 2;
            setup.hint = graphics::BufferHint::Stream;

            let handle = self.video.create_index_buffer(setup, None)?;
            self.ibo_bufs.push(handle);
        }

        for i in 0..self.idxes_strips.len() {
            let start = if i > 0 { self.idxes_strips[i - 1] } else { 0 };
            let end = self.idxes_strips[i];
            assert!((end - start) < (self.verts_chunk_num * 2));

            let slice = graphics::IndexFormat::as_bytes(&self.idxes[start..end]);
            self.video.update_index_buffer(self.ibo_bufs[i], 0, slice)?;
        }

        for i in 0..self.idxes_strips.len() {
            let start = if i > 0 { self.idxes_strips[i - 1] } else { 0 };
            let end = self.idxes_strips[i];

            let mut dc = self.video
                .make()
                .with_view(self.vso)
                .with_pipeline(self.pso)
                .with_data(self.vbo_bufs[i], Some(self.ibo_bufs[i]));

            if let Some(handle) = self.texture {
                dc = dc.with_texture("mainTexture", handle);
            }

            dc.submit(graphics::Primitive::Triangles, 0, (end - start) as u32)?;
        }

        self.verts.clear();
        self.verts_strips.clear();
        self.idxes.clear();
        self.idxes_strips.clear();

        Ok(())
    }
}