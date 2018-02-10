use std::sync::Arc;

use crayon::{application, graphics, resource, utils};
use crayon::application::errors::*;

use imgui::{DrawList, ImGui, Ui};

impl_vertex!{
    CanvasVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        color => [Color0; UByte; 4; true],
    }
}

pub struct Renderer {
    video: Arc<graphics::GraphicsSystemShared>,

    shader: graphics::ShaderHandle,
    texture: graphics::TextureHandle,

    mesh: Option<(usize, usize, graphics::MeshHandle)>,
}

impl Renderer {
    /// Creates a new `CanvasRenderer`. This will allocates essential video
    /// resources in background.
    pub fn new(ctx: &application::Context, imgui: &mut ImGui) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>();

        let layout = graphics::AttributeLayoutBuilder::new()
            .with(graphics::Attribute::Position, 2)
            .with(graphics::Attribute::Texcoord0, 2)
            .with(graphics::Attribute::Color0, 4)
            .finish();

        let mut setup = graphics::ShaderSetup::default();
        setup.layout = layout;
        setup.render_state.cull_face = graphics::CullFace::Back;
        setup.render_state.front_face_order = graphics::FrontFaceOrder::Clockwise;
        setup.render_state.color_blend = Some((
            graphics::Equation::Add,
            graphics::BlendFactor::Value(graphics::BlendValue::SourceAlpha),
            graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha),
        ));

        setup.vs = include_str!("../assets/imgui.vs").to_owned();
        setup.fs = include_str!("../assets/imgui.fs").to_owned();

        let tt = graphics::UniformVariableType::Matrix4f;
        setup.uniform_variables.insert("matrix".into(), tt);

        let tt = graphics::UniformVariableType::Texture;
        setup.uniform_variables.insert("texture".into(), tt);

        let shader = video.create_shader(resource::Location::unique(""), setup)?;

        let texture = imgui.prepare_texture(|v| {
            let mut setup = graphics::TextureSetup::default();
            setup.dimensions = (v.width as u16, v.height as u16);
            setup.filter = graphics::TextureFilter::Nearest;
            setup.format = graphics::TextureFormat::U8U8U8U8;
            video.create_texture(resource::Location::unique(""), setup, Some(v.pixels))
        })?;

        imgui.set_texture_id(**texture as usize);

        Ok(Renderer {
            video: video.clone(),
            shader: shader,
            texture: texture,
            mesh: None,
        })
    }

    pub fn render<'a>(&mut self, surface: graphics::SurfaceHandle, ui: Ui<'a>) -> Result<()> {
        ui.render(|ui, dcs| self.render_draw_list(surface, ui, &dcs))?;
        Ok(())
    }

    fn render_draw_list<'a>(
        &mut self,
        surface: graphics::SurfaceHandle,
        ui: &'a Ui<'a>,
        tasks: &DrawList<'a>,
    ) -> Result<()> {
        let mut verts = Vec::with_capacity(tasks.vtx_buffer.len());

        for v in tasks.vtx_buffer {
            let color = utils::Color::from_abgr_u32(v.col).into();
            verts.push(CanvasVertex::new(
                [v.pos.x, v.pos.y],
                [v.uv.x, v.uv.y],
                color,
            ));
        }

        let mesh = self.update_mesh(surface, &verts, &tasks.idx_buffer)?;
        let (width, height) = ui.imgui().display_size();

        if width == 0.0 || height == 0.0 {
            return Ok(());
        }

        let matrix = graphics::UniformVariable::Matrix4f(
            [
                [2.0 / width as f32, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -(height as f32), 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ],
            false,
        );

        let font_texture_id = **self.texture as usize;
        let mut idx_start = 0;
        for cmd in tasks.cmd_buffer {
            assert!(font_texture_id == cmd.texture_id as usize);

            let scissor_pos = (cmd.clip_rect.x as u16, (height - cmd.clip_rect.w) as u16);
            let scissor_size = (
                (cmd.clip_rect.z - cmd.clip_rect.x) as u16,
                (cmd.clip_rect.w - cmd.clip_rect.y) as u16,
            );

            {
                let scissor = graphics::Scissor::Enable(scissor_pos, scissor_size);
                let cmd = graphics::Command::set_scissor(scissor);
                self.video.submit(surface, 0u64, cmd)?;
            }

            {
                let mut dc = graphics::DrawCall::new(self.shader, mesh);
                dc.set_uniform_variable("matrix", matrix);
                dc.set_uniform_variable("texture", self.texture);
                let cmd = dc.build_from(idx_start, cmd.elem_count as usize)?;
                self.video.submit(surface, 0u64, cmd)?;
            }

            idx_start += cmd.elem_count as usize;
        }

        let scissor = graphics::Scissor::Disable;
        let cmd = graphics::Command::set_scissor(scissor);
        self.video.submit(surface, 0u64, cmd)?;
        Ok(())
    }

    fn update_mesh(
        &mut self,
        surface: graphics::SurfaceHandle,
        verts: &[CanvasVertex],
        idxes: &[u16],
    ) -> Result<graphics::MeshHandle> {
        if let Some((nv, ni, handle)) = self.mesh {
            if nv >= verts.len() && ni >= idxes.len() {
                let slice = CanvasVertex::encode(verts);
                let cmd = graphics::Command::update_vertex_buffer(handle, 0, slice);
                self.video.submit(surface, 0u64, cmd)?;

                let slice = graphics::IndexFormat::encode(idxes);
                let cmd = graphics::Command::update_index_buffer(handle, 0, slice);
                self.video.submit(surface, 0u64, cmd)?;

                return Ok(handle);
            }

            self.video.delete_mesh(handle);
        }

        let mut nv = 1;
        while nv < verts.len() {
            nv *= 2;
        }

        let mut ni = 1;
        while ni < idxes.len() {
            ni *= 2;
        }

        let mut setup = graphics::MeshSetup::default();
        setup.hint = graphics::BufferHint::Stream;
        setup.layout = CanvasVertex::layout();
        setup.index_format = graphics::IndexFormat::U16;
        setup.primitive = graphics::Primitive::Triangles;
        setup.num_verts = nv;
        setup.num_idxes = ni;

        let verts_slice = CanvasVertex::encode(verts);
        let idxes_slice = graphics::IndexFormat::encode(idxes);
        let mesh = self.video.create_mesh(
            resource::Location::unique(""),
            setup,
            verts_slice,
            idxes_slice,
        )?;
        self.mesh = Some((nv, ni, mesh));
        Ok(mesh)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.video.delete_shader(self.shader);
        self.video.delete_texture(self.texture);

        if let Some((_, _, mesh)) = self.mesh.take() {
            self.video.delete_mesh(mesh);
        }
    }
}
