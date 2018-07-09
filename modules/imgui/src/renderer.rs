use std::sync::Arc;

use crayon::{application, math};

use crayon::application::window;
use crayon::video::assets::prelude::*;
use crayon::video::errors::*;
use crayon::video::prelude::*;

use imgui::{DrawList, ImGui, Ui};

impl_vertex!{
    CanvasVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        color => [Color0; UByte; 4; true],
    }
}

pub struct Renderer {
    video: Arc<VideoSystemShared>,
    window: Arc<window::WindowShared>,

    shader: ShaderHandle,
    texture: TextureHandle,

    mesh: Option<(usize, usize, MeshHandle)>,
}

impl Renderer {
    /// Creates a new `CanvasRenderer`. This will allocates essential video
    /// resources in background.
    pub fn new(ctx: &application::Context, imgui: &mut ImGui) -> Result<Self> {
        let layout = AttributeLayout::build()
            .with(Attribute::Position, 2)
            .with(Attribute::Texcoord0, 2)
            .with(Attribute::Color0, 4)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("matrix", UniformVariableType::Matrix4f)
            .with("texture", UniformVariableType::Texture)
            .finish();

        let mut render_state = RenderState::default();
        render_state.cull_face = CullFace::Back;
        render_state.front_face_order = FrontFaceOrder::Clockwise;
        render_state.color_blend = Some((
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        ));

        let mut setup = ShaderSetup::default();
        setup.params.attributes = layout;
        setup.params.uniforms = uniforms;
        setup.params.render_state = render_state;
        setup.vs = include_str!("../assets/imgui.vs").to_owned();
        setup.fs = include_str!("../assets/imgui.fs").to_owned();
        let shader = ctx.video.create_shader(setup)?;

        let texture = imgui.prepare_texture(|v| {
            let mut setup = TextureSetup::default();
            setup.params.dimensions = (v.width, v.height).into();
            setup.params.filter = TextureFilter::Nearest;
            setup.params.format = TextureFormat::U8U8U8U8;
            setup.data = Some(v.pixels);
            ctx.video.create_texture(setup)
        })?;

        imgui.set_texture_id(**texture as usize);

        Ok(Renderer {
            video: ctx.video.clone(),
            window: ctx.window.clone(),

            shader: shader,
            texture: texture,
            mesh: None,
        })
    }

    pub fn render<'a>(&mut self, surface: SurfaceHandle, ui: Ui<'a>) -> Result<()> {
        ui.render(|ui, dcs| self.render_draw_list(surface, ui, &dcs))?;
        Ok(())
    }

    fn render_draw_list<'a>(
        &mut self,
        surface: SurfaceHandle,
        ui: &'a Ui<'a>,
        tasks: &DrawList<'a>,
    ) -> Result<()> {
        let mut verts = Vec::with_capacity(tasks.vtx_buffer.len());

        for v in tasks.vtx_buffer {
            let color = math::Color::from_abgr_u32(v.col).into();
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

        let matrix = UniformVariable::Matrix4f(
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
        let hidpi = self.window.hidpi();

        for cmd in tasks.cmd_buffer {
            assert!(font_texture_id == cmd.texture_id as usize);

            let scissor_pos = math::Vector2::new(
                (cmd.clip_rect.x as f32 * hidpi) as i32,
                ((height - cmd.clip_rect.w) as f32 * hidpi) as i32,
            );

            let scissor_size = math::Vector2::new(
                ((cmd.clip_rect.z - cmd.clip_rect.x) as f32 * hidpi) as u32,
                ((cmd.clip_rect.w - cmd.clip_rect.y) as f32 * hidpi) as u32,
            );

            {
                let scissor = SurfaceScissor::Enable {
                    position: scissor_pos,
                    size: scissor_size,
                };
                let cmd = Command::set_scissor(scissor);
                self.video.submit(surface, 0u64, cmd)?;
            }

            {
                let mut dc = DrawCall::new(self.shader, mesh);
                dc.set_uniform_variable("matrix", matrix);
                dc.set_uniform_variable("texture", self.texture);
                let cmd = dc.build_from(idx_start, cmd.elem_count as usize)?;
                self.video.submit(surface, 0u64, cmd)?;
            }

            idx_start += cmd.elem_count as usize;
        }

        let scissor = SurfaceScissor::Disable;
        let cmd = Command::set_scissor(scissor);
        self.video.submit(surface, 0u64, cmd)?;
        Ok(())
    }

    fn update_mesh(
        &mut self,
        surface: SurfaceHandle,
        verts: &[CanvasVertex],
        idxes: &[u16],
    ) -> Result<MeshHandle> {
        if let Some((nv, ni, handle)) = self.mesh {
            if nv >= verts.len() && ni >= idxes.len() {
                let slice = CanvasVertex::encode(verts);
                let cmd = Command::update_vertex_buffer(handle, 0, slice);
                self.video.submit(surface, 0u64, cmd)?;

                let slice = IndexFormat::encode(idxes);
                let cmd = Command::update_index_buffer(handle, 0, slice);
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

        let mut setup = MeshSetup::default();
        setup.params.hint = MeshHint::Stream;
        setup.params.layout = CanvasVertex::layout();
        setup.params.index_format = IndexFormat::U16;
        setup.params.primitive = MeshPrimitive::Triangles;
        setup.params.num_verts = nv;
        setup.params.num_idxes = ni;
        setup.verts = Some(CanvasVertex::encode(verts));
        setup.idxes = Some(IndexFormat::encode(idxes));

        let mesh = self.video.create_mesh(setup)?;
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
