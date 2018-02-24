use crayon::prelude::*;
use crayon::graphics::assets::prelude::*;
use crayon::graphics::GraphicsSystemGuard;
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
    video: GraphicsSystemGuard,
    pass: Pass,
    post_effect: Pass,
    texture: RenderTextureHandle,
    time: f32,
}

impl Window {
    pub fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();
        let mut video = GraphicsSystemGuard::new(ctx.shared::<GraphicsSystem>().clone());

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
            let mut setup = MeshSetup::default();
            setup.num_verts = 3;
            setup.num_idxes = 3;
            setup.layout = Vertex::layout();

            let mesh = video.create_mesh(
                Location::unique(""),
                setup,
                Vertex::encode(&verts[..]),
                IndexFormat::encode(&idxes),
            )?;

            // Create render texture for post effect.
            let mut setup = RenderTextureSetup::default();
            setup.format = RenderTextureFormat::RGBA8;
            setup.dimensions = (568, 320);
            let rendered_texture = video.create_render_texture(setup)?;

            // Create the surface state for pass 1.
            let mut setup = SurfaceSetup::default();
            setup.set_attachments(&[rendered_texture], None)?;
            setup.set_order(0);
            setup.set_clear(Color::gray(), None, None);
            let surface = video.create_surface(setup)?;

            // Create shader state.
            let mut setup = ShaderSetup::default();
            setup.layout = attributes;
            setup.vs = include_str!("../../assets/render_target_p1.vs").to_owned();
            setup.fs = include_str!("../../assets/render_target_p1.fs").to_owned();
            let shader = video.create_shader(Location::unique(""), setup)?;

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

            let mut setup = MeshSetup::default();
            setup.num_verts = 4;
            setup.num_idxes = 6;
            setup.layout = Vertex::layout();

            let mesh = video.create_mesh(
                Location::unique(""),
                setup,
                Vertex::encode(&verts[..]),
                IndexFormat::encode(&idxes),
            )?;

            let mut setup = SurfaceSetup::default();
            setup.set_order(1);
            let surface = video.create_surface(setup)?;

            let mut setup = ShaderSetup::default();
            setup.layout = attributes;
            setup.vs = include_str!("../../assets/render_target_p2.vs").to_owned();
            setup.fs = include_str!("../../assets/render_target_p2.fs").to_owned();
            let tt = UniformVariableType::RenderTexture;
            setup.uniform_variables.insert("renderedTexture".into(), tt);
            let tt = UniformVariableType::F32;
            setup.uniform_variables.insert("time".into(), tt);
            let shader = video.create_shader(Location::unique(""), setup)?;

            Pass {
                surface: surface,
                shader: shader,
                mesh: mesh,
            }
        };

        Ok(Window {
            video: video,

            pass: pass,
            post_effect: post_effect,
            texture: rendered_texture,

            time: 0.0,
        })
    }
}

impl Application for Window {
    type Error = Error;

    fn on_update(&mut self, _: &Context) -> Result<()> {
        {
            let mut dc = DrawCall::new(self.pass.shader, self.pass.mesh);
            let cmd = dc.build_from(0, 3)?;
            self.video.submit(self.pass.surface, 0u64, cmd)?;
        }

        {
            let mut dc = DrawCall::new(self.post_effect.shader, self.post_effect.mesh);
            dc.set_uniform_variable("renderedTexture", self.texture);
            dc.set_uniform_variable("time", self.time);
            let cmd = dc.build_from(0, 6)?;
            self.video.submit(self.post_effect.surface, 1u64, cmd)?;
        }

        self.time += 0.05;
        Ok(())
    }
}

pub fn main(mut settings: Settings) {
    settings.window.width = 568;
    settings.window.height = 320;

    let mut engine = Engine::new_with(&settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
