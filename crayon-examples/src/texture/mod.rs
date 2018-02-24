use crayon::prelude::*;
use crayon::graphics::assets::prelude::*;
use crayon::graphics::GraphicsSystemGuard;

use utils::*;
use errors::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    surface: SurfaceHandle,
    shader: ShaderHandle,
    mesh: MeshHandle,
    texture: TextureHandle,
    video: GraphicsSystemGuard,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let assets = format!("{0}/assets", env!("CARGO_MANIFEST_DIR"));
        engine.resource.mount("std", DirectoryFS::new(assets)?)?;

        let ctx = engine.context();
        let mut video = GraphicsSystemGuard::new(ctx.shared::<GraphicsSystem>().clone());

        let verts: [Vertex; 4] = [
            Vertex::new([-1.0, -1.0]),
            Vertex::new([1.0, -1.0]),
            Vertex::new([1.0, 1.0]),
            Vertex::new([-1.0, 1.0]),
        ];
        let idxes: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let attributes = AttributeLayoutBuilder::new()
            .with(Attribute::Position, 2)
            .finish();

        // Create vertex buffer object.
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

        // Create the view state.
        let setup = SurfaceSetup::default();
        let surface = video.create_surface(setup)?;

        // Create shader state.
        let mut setup = ShaderSetup::default();
        setup.layout = attributes;
        setup.vs = include_str!("../../assets/texture.vs").to_owned();
        setup.fs = include_str!("../../assets/texture.fs").to_owned();
        let tt = UniformVariableType::Texture;
        setup.uniform_variables.insert("renderedTexture".into(), tt);
        let shader = video.create_shader(Location::unique(""), setup)?;

        let setup = TextureSetup::default();
        let location = Location::unique("/std/texture.png");
        let texture = video
            .create_texture_from::<TextureParser>(location, setup)
            .unwrap();

        Ok(Window {
            surface: surface,
            shader: shader,
            mesh: mesh,
            texture: texture,
            video: video,
        })
    }
}

impl Application for Window {
    type Error = Error;

    fn on_update(&mut self, _: &Context) -> Result<()> {
        let mut dc = DrawCall::new(self.shader, self.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        let cmd = dc.build_from(0, 6)?;
        self.video.submit(self.surface, 0u64, cmd)?;
        Ok(())
    }
}

pub fn main(mut settings: Settings) {
    settings.window.width = 232;
    settings.window.height = 217;

    let mut engine = Engine::new_with(&settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
