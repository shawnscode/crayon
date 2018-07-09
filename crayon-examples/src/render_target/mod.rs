use crayon::prelude::*;
use crayon::video::assets::prelude::*;
use errors::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Pass {
    surface: SurfaceHandle,
    shader: ShaderHandle,
    mesh: MeshHandle,
}

struct Window {
    video: VideoSystemGuard,

    pass: Pass,
    post_effect: Pass,

    batch: Batch,
    texture: RenderTextureHandle,
    time: f32,
}

impl Window {
    pub fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();
        let mut video = VideoSystemGuard::new(ctx.video.clone());

        let attributes = AttributeLayoutBuilder::new()
            .with(Attribute::Position, 2)
            .finish();

        //
        let (pass, rendered_texture) = {
            let verts: [Vertex; 3] = [
                Vertex::new([0.0, 0.5]),
                Vertex::new([0.5, -0.5]),
                Vertex::new([-0.5, -0.5]),
            ];
            let idxes: [u16; 3] = [0, 1, 2];

            // Create vertex buffer object.
            let mut params = MeshParams::default();
            params.num_verts = 3;
            params.num_idxes = 3;
            params.layout = Vertex::layout();
            let vptr = Some(Vertex::encode(&verts[..]));
            let iptr = Some(IndexFormat::encode(&idxes));
            let mesh = video.create_mesh(params, vptr, iptr)?;

            // Create render texture for post effect.
            let mut params = RenderTextureParams::default();
            params.format = RenderTextureFormat::RGBA8;
            params.dimensions = (568, 320).into();
            let rendered_texture = video.create_render_texture(params)?;

            // Create the surface state for pass 1.
            let mut params = SurfaceParams::default();
            params.set_attachments(&[rendered_texture], None)?;
            params.set_clear(math::Color::gray(), None, None);
            let surface = video.create_surface(params)?;

            // Create shader state.
            let mut params = ShaderParams::default();
            params.attributes = attributes;
            let vs = include_str!("../../assets/render_target_p1.vs").to_owned();
            let fs = include_str!("../../assets/render_target_p1.fs").to_owned();
            let shader = video.create_shader(params, vs, fs)?;

            (
                Pass {
                    surface: surface,
                    shader: shader,
                    mesh: mesh,
                },
                rendered_texture,
            )
        };

        let post_effect = {
            let verts: [Vertex; 4] = [
                Vertex::new([-1.0, -1.0]),
                Vertex::new([1.0, -1.0]),
                Vertex::new([1.0, 1.0]),
                Vertex::new([-1.0, 1.0]),
            ];
            let idxes: [u16; 6] = [0, 1, 2, 0, 2, 3];

            let mut params = MeshParams::default();
            params.num_verts = 4;
            params.num_idxes = 6;
            params.layout = Vertex::layout();

            let vptr = Some(Vertex::encode(&verts[..]));
            let iptr = Some(IndexFormat::encode(&idxes));
            let mesh = video.create_mesh(params, vptr, iptr)?;

            let params = SurfaceParams::default();
            let surface = video.create_surface(params)?;

            let uniforms = UniformVariableLayout::build()
                .with("renderedTexture", UniformVariableType::RenderTexture)
                .with("time", UniformVariableType::F32)
                .finish();

            let mut params = ShaderParams::default();
            params.attributes = attributes;
            params.uniforms = uniforms;
            let vs = include_str!("../../assets/render_target_p2.vs").to_owned();
            let fs = include_str!("../../assets/render_target_p2.fs").to_owned();
            let shader = video.create_shader(params, vs, fs)?;

            Pass {
                surface: surface,
                shader: shader,
                mesh: mesh,
            }
        };

        Ok(Window {
            video: video,

            batch: Batch::new(),
            pass: pass,
            post_effect: post_effect,
            texture: rendered_texture,

            time: 0.0,
        })
    }
}

impl Application for Window {
    type Error = Error;

    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let surface = self.pass.surface;
        let dc = DrawCall::new(self.pass.shader, self.pass.mesh);
        self.batch.draw(dc);
        self.batch.submit(&ctx.video, surface)?;

        let surface = self.post_effect.surface;
        let mut dc = DrawCall::new(self.post_effect.shader, self.post_effect.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        dc.set_uniform_variable("time", self.time);
        self.batch.draw(dc);
        self.batch.submit(&ctx.video, surface)?;

        self.time += 0.05;
        Ok(())
    }
}

pub fn main(mut settings: Settings) {
    settings.window.size = math::Vector2::new(568, 320);

    let mut engine = Engine::new_with(&settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
