use crayon::prelude::*;
use utils::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    _label: graphics::RAIIGuard,

    surface: graphics::SurfaceHandle,
    shader: graphics::ShaderHandle,
    mesh: graphics::MeshHandle,
    texture: graphics::TextureHandle,
}

impl Window {
    fn new(engine: &mut Engine) -> errors::Result<Self> {
        engine
            .resource
            .mount("std", resource::filesystem::DirectoryFS::new("resources")?)?;

        let ctx = engine.context();
        let video = ctx.shared::<GraphicsSystem>().clone();
        let mut label = graphics::RAIIGuard::new(video);

        let verts: [Vertex; 4] = [Vertex::new([-1.0, -1.0]),
                                  Vertex::new([1.0, -1.0]),
                                  Vertex::new([1.0, 1.0]),
                                  Vertex::new([-1.0, 1.0])];
        let idxes: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let attributes = graphics::AttributeLayoutBuilder::new()
            .with(graphics::Attribute::Position, 2)
            .finish();

        // Create vertex buffer object.
        let mut setup = graphics::MeshSetup::default();
        setup.num_vertices = 4;
        setup.num_indices = 6;
        setup.layout = Vertex::layout();

        let mesh = label
            .create_mesh(setup,
                         Vertex::as_bytes(&verts[..]),
                         graphics::IndexFormat::as_bytes(&idxes))?;

        // Create the view state.
        let setup = graphics::SurfaceSetup::default();
        let surface = label.create_surface(setup)?;

        // Create shader state.
        let mut setup = graphics::ShaderSetup::default();
        setup.layout = attributes;
        setup.vs = include_str!("../../resources/texture.vs").to_owned();
        setup.fs = include_str!("../../resources/texture.fs").to_owned();
        setup.uniform_variables.push("renderedTexture".into());
        let shader = label.create_shader(setup)?;

        let setup = graphics::TextureSetup::default();
        let location = Location::unique("/std/texture.png");
        let texture = label
            .create_texture_from::<TextureParser>(location, setup)
            .unwrap();

        Ok(Window {
               surface: surface,
               shader: shader,
               mesh: mesh,
               texture: texture,
               _label: label,
           })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        let video = ctx.shared::<GraphicsSystem>();

        let mut dc = graphics::DrawCall::new(self.shader, self.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        let cmd = dc.build(0, 6)?;
        video.submit(self.surface, 0, cmd)?;

        Ok(())
    }
}

pub fn main(title: String, _: &[String]) {
    let mut settings = Settings::default();
    settings.window.width = 232;
    settings.window.height = 217;
    settings.window.title = title;

    let mut engine = Engine::new_with(settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}