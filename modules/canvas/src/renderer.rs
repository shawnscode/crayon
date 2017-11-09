use crayon::{graphics, application, math};
use errors::*;

impl_vertex!{
    CanvasVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        color => [Color0; UByte; 4; true],
    }
}

pub struct CanvasRenderer {
    vso: graphics::ViewStateHandle,
    pso: graphics::PipelineStateHandle,

    vbo_bufs: Vec<graphics::VertexBufferHandle>,
    ibo_bufs: Vec<graphics::IndexBufferHandle>,

    verts: Vec<CanvasVertex>,
    verts_strips: Vec<usize>,
    verts_chunk_num: usize,

    idxes: Vec<u16>,
    idxes_strips: Vec<usize>,

    text_renderer: CanvasTextRenderer,
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
               vso: vso,
               pso: pso,

               vbo_bufs: Vec::new(),
               ibo_bufs: Vec::new(),

               verts: Vec::new(),
               verts_strips: Vec::new(),
               verts_chunk_num: 4096,

               idxes: Vec::new(),
               idxes_strips: Vec::new(),

               text_renderer: CanvasTextRenderer::new(ctx)?,
           })
    }

    pub fn clear(&mut self, ctx: &application::Context) {
        let video = ctx.shared::<graphics::GraphicsSystem>();

        for handle in self.vbo_bufs.drain(..) {
            video.delete_vertex_buffer(handle);
        }

        for handle in self.ibo_bufs.drain(..) {
            video.delete_index_buffer(handle);
        }
    }

    pub fn draw_text(&mut self, text: &str) {
        let (verts, idxes) = self.text_renderer.draw(text);
        self.add_mesh(&verts, &idxes);
    }

    pub fn add_mesh(&mut self, verts: &[CanvasVertex], idxes: &[u16]) {
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

    pub fn flush(&mut self, ctx: &application::Context) -> Result<()> {
        assert!(self.idxes_strips.len() == self.verts_strips.len());

        if self.idxes.len() <= 0 {
            return Ok(());
        }

        let video = ctx.shared::<graphics::GraphicsSystem>();

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

            let handle = video.create_vertex_buffer(setup, None)?;
            self.vbo_bufs.push(handle);
        }

        for i in 0..self.verts_strips.len() {
            let start = if i > 0 { self.verts_strips[i - 1] } else { 0 };
            let end = self.verts_strips[i];
            assert!((end - start) < self.verts_chunk_num);

            let slice = CanvasVertex::as_bytes(&self.verts[start..end]);
            video.update_vertex_buffer(self.vbo_bufs[i], 0, slice)?;
        }

        for _ in self.ibo_bufs.len()..self.idxes_strips.len() {
            let mut setup = graphics::IndexBufferSetup::default();
            setup.format = graphics::IndexFormat::U16;
            setup.num = self.verts_chunk_num * 2;
            setup.hint = graphics::BufferHint::Stream;

            let handle = video.create_index_buffer(setup, None)?;
            self.ibo_bufs.push(handle);
        }

        for i in 0..self.idxes_strips.len() {
            let start = if i > 0 { self.idxes_strips[i - 1] } else { 0 };
            let end = self.idxes_strips[i];
            assert!((end - start) < (self.verts_chunk_num * 2));

            let slice = graphics::IndexFormat::as_bytes(&self.idxes[start..end]);
            video.update_index_buffer(self.ibo_bufs[i], 0, slice)?;
        }

        for i in 0..self.idxes_strips.len() {
            let start = if i > 0 { self.idxes_strips[i - 1] } else { 0 };
            let end = self.idxes_strips[i];

            video
                .make()
                .with_view(self.vso)
                .with_pipeline(self.pso)
                .with_data(self.vbo_bufs[i], Some(self.ibo_bufs[i]))
                .with_texture("tex", self.text_renderer.texture())
                .submit(graphics::Primitive::Triangles, 0, (end - start) as u32)?;
        }

        self.verts.clear();
        self.verts_strips.clear();
        self.idxes.clear();
        self.idxes_strips.clear();

        Ok(())
    }
}

use rusttype;
use std::sync::Arc;

pub struct CanvasTextRenderer {
    font: rusttype::Font<'static>,
    texture: graphics::TextureHandle,
    cache: rusttype::gpu_cache::Cache,
    video: Arc<graphics::GraphicsSystemShared>,
}

impl CanvasTextRenderer {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let data = include_bytes!("../assets/fonts/FiraSans-Regular.ttf");
        let collection = rusttype::FontCollection::from_bytes(data as &[u8]);
        let font = collection.into_font().unwrap();

        let video = ctx.shared::<graphics::GraphicsSystem>().clone();
        let cache = rusttype::gpu_cache::Cache::new(512, 512, 0.1, 0.1);

        let mut setup = graphics::TextureSetup::default();
        setup.filter = graphics::TextureFilter::Linear;
        setup.mipmap = false;
        setup.dimensions = (512, 512);
        setup.format = graphics::TextureFormat::U8;
        let texture = video.create_texture(setup, None)?;

        Ok(CanvasTextRenderer {
               font: font,
               texture: texture,
               cache: cache,
               video: video,
           })
    }

    pub fn texture(&self) -> graphics::TextureHandle {
        self.texture
    }

    pub fn draw(&mut self, text: &str) -> (Vec<CanvasVertex>, Vec<u16>) {
        let glyphs = Self::layout(&self.font, rusttype::Scale::uniform(144.0), 480, text);
        for v in &glyphs {
            self.cache.queue_glyph(0, v.clone());
        }

        let handle = self.texture;
        let video = self.video.clone();

        let dimensions = (640f32, 480f32);
        self.cache
            .cache_queued(|rect, data| {
                              let rect = graphics::Rect::new(math::Point2::new(rect.min.x as i32,
                                                                               rect.min.y as i32),
                                                             math::Point2::new(rect.max.x as i32,
                                                                               rect.max.y as i32));

                              video.update_texture(handle, rect, data);
                          })
            .unwrap();

        let mut verts = Vec::new();
        let mut idxes = Vec::new();
        let color = [0, 0, 0, 255];
        for v in &glyphs {
            if let Ok(Some((uv, screen))) = self.cache.rect_for(0, v) {
                let min = (math::Vector2::new(screen.min.x as f32 / dimensions.0 - 0.5,
                                              1.0 - screen.min.y as f32 / dimensions.1 - 0.5)) *
                          2.0;
                let max = (math::Vector2::new(screen.max.x as f32 / dimensions.0 - 0.5,
                                              1.0 - screen.max.y as f32 / dimensions.1 - 0.5)) *
                          2.0;

                let offset = verts.len() as u16;
                verts.push(CanvasVertex::new([min.x, max.y], [uv.min.x, uv.max.y], color));
                verts.push(CanvasVertex::new([min.x, min.y], [uv.min.x, uv.min.y], color));
                verts.push(CanvasVertex::new([max.x, min.y], [uv.max.x, uv.min.y], color));
                verts.push(CanvasVertex::new([max.x, max.y], [uv.max.x, uv.max.y], color));

                idxes.push(offset + 0);
                idxes.push(offset + 1);
                idxes.push(offset + 2);
                idxes.push(offset + 2);
                idxes.push(offset + 3);
                idxes.push(offset + 0);
            }
        }

        (verts, idxes)
    }

    pub fn layout<'a>(font: &'a rusttype::Font<'a>,
                      scale: rusttype::Scale,
                      width: u32,
                      text: &str)
                      -> Vec<rusttype::PositionedGlyph<'a>> {
        let mut result = Vec::new();

        let v_metrics = font.v_metrics(scale);
        let line = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        let mut caret = rusttype::point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;

        for c in text.chars() {
            if let Some(glyph) = font.glyph(c) {
                if let Some(id) = last_glyph_id.take() {
                    caret.x += font.pair_kerning(scale, id, glyph.id());
                }

                last_glyph_id = Some(glyph.id());

                let mut glyph = glyph.scaled(scale).positioned(caret);
                if let Some(bb) = glyph.pixel_bounding_box() {
                    if bb.max.x > width as i32 {
                        caret = rusttype::point(0.0, caret.y + line);
                        glyph = glyph.into_unpositioned().positioned(caret);
                        last_glyph_id = None;
                    }
                }

                caret.x += glyph.unpositioned().h_metrics().advance_width;
                result.push(glyph);
            }
        }

        result
    }
}