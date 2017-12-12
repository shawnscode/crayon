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
    vbo: graphics::VertexBufferHandle,
    texture: graphics::TextureHandle,
}

impl Window {
    fn new(engine: &mut Engine) -> errors::Result<Self> {
        engine
            .resource
            .mount("std", resource::filesystem::DirectoryFS::new("resources")?)?;

        let ctx = engine.context();
        let ctx = ctx.read().unwrap();
        let video = ctx.shared::<GraphicsSystem>().clone();
        let mut label = graphics::RAIIGuard::new(video);

        let verts: [Vertex; 6] = [Vertex::new([-1.0, -1.0]),
                                  Vertex::new([1.0, -1.0]),
                                  Vertex::new([-1.0, 1.0]),
                                  Vertex::new([-1.0, 1.0]),
                                  Vertex::new([1.0, -1.0]),
                                  Vertex::new([1.0, 1.0])];

        let attributes = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .finish();

        // Create vertex buffer object.
        let mut setup = graphics::VertexBufferSetup::default();
        setup.num = verts.len();
        setup.layout = Vertex::layout();
        let vbo = label
            .create_vertex_buffer(setup, Some(Vertex::as_bytes(&verts[..])))?;

        // Create the view state.
        let setup = graphics::SurfaceSetup::default();
        let surface = label.create_view(setup)?;

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
               vbo: vbo,
               texture: texture,
               _label: label,
           })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        let video = ctx.shared::<GraphicsSystem>();

        let mut dc = DrawCall::new(self.shader);
        dc.set_mesh(self.vbo, None);
        dc.set_uniform_variable("renderedTexture", self.texture);
        let task = dc.draw(graphics::Primitive::Triangles, 0, 6)?;
        video.submit(self.surface, task)?;

        Ok(())
    }
}

pub fn main(_: &[String]) {
    let mut settings = Settings::default();
    settings.window.width = 232;
    settings.window.height = 217;

    let mut engine = Engine::new_with(settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}