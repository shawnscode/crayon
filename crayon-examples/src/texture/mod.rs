use crayon::prelude::*;
use utils::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    _label: graphics::utils::RAIIGuard,

    vso: graphics::ViewStateHandle,
    pso: graphics::PipelineStateHandle,
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
        let mut label = graphics::utils::RAIIGuard::new(video);

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
        let setup = graphics::ViewStateSetup::default();
        let vso = label.create_view(setup)?;

        // Create pipeline state.
        let mut setup = graphics::PipelineStateSetup::default();
        setup.layout = attributes;
        let vs = include_str!("../../resources/texture.vs").to_owned();
        let fs = include_str!("../../resources/texture.fs").to_owned();
        let pso = label.create_pipeline(setup, vs, fs)?;

        let setup = graphics::TextureSetup::default();
        let location = Location::unique("/std/texture.png");
        let texture = label
            .create_texture_from::<TextureParser>(location, setup)
            .unwrap();

        Ok(Window {
               vso: vso,
               pso: pso,
               vbo: vbo,
               texture: texture,
               _label: label,
           })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        ctx.shared::<GraphicsSystem>()
            .make()
            .with_view(self.vso)
            .with_pipeline(self.pso)
            .with_data(self.vbo, None)
            .with_texture("renderedTexture", self.texture)
            .submit(graphics::Primitive::Triangles, 0, 6)?;

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