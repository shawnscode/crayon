use std::sync::Arc;

use crayon::{graphics, application, utils};

use imgui::{DrawList, ImGui, Ui};
use errors::*;

impl_vertex!{
    CanvasVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        color => [Color0; UByte; 4; true],
    }
}

pub struct Renderer {
    video: Arc<graphics::GraphicsSystemShared>,

    surface: graphics::SurfaceHandle,
    shader: graphics::ShaderHandle,
    texture: graphics::TextureHandle,

    vbo: Option<(u32, graphics::VertexBufferHandle)>,
    ibo: Option<(u32, graphics::IndexBufferHandle)>,
}

impl Renderer {
    /// Creates a new `CanvasRenderer`. This will allocates essential video
    /// resources in background.
    pub fn new(ctx: &application::Context, imgui: &mut ImGui) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>();

        let mut setup = graphics::SurfaceSetup::default();
        setup.clear_color = utils::Color::white().into();
        let surface = video.create_surface(setup)?;

        let layout = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .with(graphics::VertexAttribute::Texcoord0, 2)
            .with(graphics::VertexAttribute::Color0, 4)
            .finish();

        let mut setup = graphics::ShaderSetup::default();
        setup.layout = layout;
        setup.render_state.cull_face = graphics::CullFace::Back;
        setup.render_state.front_face_order = graphics::FrontFaceOrder::Clockwise;
        setup.render_state.color_blend =
            Some((graphics::Equation::Add,
                  graphics::BlendFactor::Value(graphics::BlendValue::SourceAlpha),
                  graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha)));

        setup.vs = include_str!("../resources/imgui.vs").to_owned();
        setup.fs = include_str!("../resources/imgui.fs").to_owned();
        setup.uniform_variables.push("matrix".into());
        setup.uniform_variables.push("texture".into());
        let shader = video.create_shader(setup)?;

        let texture = imgui
            .prepare_texture(|v| {
                                 let mut setup = graphics::TextureSetup::default();
                                 setup.dimensions = (v.width, v.height);
                                 setup.filter = graphics::TextureFilter::Nearest;
                                 setup.format = graphics::TextureFormat::U8U8U8U8;
                                 video.create_texture(setup, Some(v.pixels))
                             })?;

        imgui.set_texture_id(**texture as usize);

        Ok(Renderer {
               video: video.clone(),

               surface: surface,
               shader: shader,
               texture: texture,
               vbo: None,
               ibo: None,
           })
    }

    pub fn render<'a>(&mut self, ui: Ui<'a>) -> Result<()> {
        ui.render(|ui, dcs| self.render_draw_list(ui, &dcs))
    }

    fn render_draw_list<'a>(&mut self, ui: &'a Ui<'a>, tasks: &DrawList<'a>) -> Result<()> {
        let mut verts = Vec::with_capacity(tasks.vtx_buffer.len());

        for v in tasks.vtx_buffer {
            let color = utils::Color::from_abgr_u32(v.col).into();
            verts.push(CanvasVertex::new([v.pos.x, v.pos.y], [v.uv.x, v.uv.y], color));
        }

        let vbo = self.update_vertex_buffer(&verts)?;
        let ibo = self.update_index_buffer(&tasks.idx_buffer)?;

        let (width, height) = ui.imgui().display_size();
        let (scale_width, scale_height) = ui.imgui().display_framebuffer_scale();

        if width == 0.0 || height == 0.0 {
            return Ok(());
        }

        let matrix = graphics::UniformVariable::Matrix4f([[2.0 / width as f32, 0.0, 0.0, 0.0],
                                                          [0.0, 2.0 / -(height as f32), 0.0, 0.0],
                                                          [0.0, 0.0, -1.0, 0.0],
                                                          [-1.0, 1.0, 0.0, 1.0]],
                                                         false);

        let font_texture_id = **self.texture as usize;
        let mut idx_start = 0;
        for cmd in tasks.cmd_buffer {
            assert!(font_texture_id == cmd.texture_id as usize);

            let scissor_pos = ((cmd.clip_rect.x * scale_width) as u16,
                               ((height - cmd.clip_rect.w) * scale_height) as u16);
            let scissor_size = (((cmd.clip_rect.z - cmd.clip_rect.x) * scale_width) as u16,
                                ((cmd.clip_rect.w - cmd.clip_rect.y) * scale_height) as u16);

            let scissor = graphics::Scissor::Enable(scissor_pos, scissor_size);
            let task = graphics::BucketTask::set_scissor(scissor);
            self.video.submit(self.surface, task)?;

            let mut dc = graphics::DrawCall::new(self.shader);
            dc.set_uniform_variable("matrix", matrix);
            dc.set_uniform_variable("texture", self.texture);
            dc.set_mesh(vbo, ibo);

            let task = dc.draw(graphics::Primitive::Triangles, idx_start, cmd.elem_count)?;
            self.video.submit(self.surface, task)?;

            idx_start += cmd.elem_count;
        }

        Ok(())
    }

    fn update_vertex_buffer(&mut self,
                            vertices: &[CanvasVertex])
                            -> Result<graphics::VertexBufferHandle> {
        if let Some((num, handle)) = self.vbo {
            if num >= vertices.len() as u32 {
                let slice = CanvasVertex::as_bytes(vertices);
                let task = graphics::BucketTask::update_vertex_buffer(handle, 0, slice);
                self.video.submit(self.surface, task)?;
                return Ok(handle);
            }

            self.video.delete_vertex_buffer(handle);
        }

        let mut num = 1;
        while num < vertices.len() as u32 {
            num *= 2;
        }

        let mut setup = graphics::VertexBufferSetup::default();
        setup.layout = CanvasVertex::layout();
        setup.num = num;
        setup.hint = graphics::BufferHint::Stream;

        let slice = CanvasVertex::as_bytes(vertices);
        let vbo = self.video.create_vertex_buffer(setup, Some(slice))?;

        self.vbo = Some((num, vbo));
        Ok(vbo)
    }

    fn update_index_buffer(&mut self, indices: &[u16]) -> Result<graphics::IndexBufferHandle> {
        if let Some((num, handle)) = self.ibo {
            if num >= indices.len() as u32 {
                let slice = graphics::IndexFormat::as_bytes(indices);
                let task = graphics::BucketTask::update_index_buffer(handle, 0, slice);
                self.video.submit(self.surface, task)?;
                return Ok(handle);
            }

            self.video.delete_index_buffer(handle);
        }

        let mut num = 1;
        while num < indices.len() as u32 {
            num *= 2;
        }

        let mut setup = graphics::IndexBufferSetup::default();
        setup.format = graphics::IndexFormat::U16;
        setup.num = num;
        setup.hint = graphics::BufferHint::Stream;

        let slice = graphics::IndexFormat::as_bytes(indices);
        let ibo = self.video.create_index_buffer(setup, Some(slice))?;

        self.ibo = Some((num, ibo));
        Ok(ibo)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.video.delete_surface(self.surface);
        self.video.delete_shader(self.shader);
        self.video.delete_texture(self.texture);

        if let Some((_, vbo)) = self.vbo {
            self.video.delete_vertex_buffer(vbo);
        }

        if let Some((_, ibo)) = self.ibo {
            self.video.delete_index_buffer(ibo);
        }
    }
}