#[macro_use]
extern crate crayon;
extern crate crayon_workflow;

mod utils;

use crayon::prelude::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    view: graphics::ViewStateRef,
    pso: graphics::PipelineStateRef,
    vbo: graphics::VertexBufferRef,
    texture: TexturePtr,
}

impl Window {
    fn new(app: &mut Application) -> errors::Result<Self> {
        let quad_vertices: [Vertex; 6] = [Vertex::new([-1.0, -1.0]),
                                          Vertex::new([1.0, -1.0]),
                                          Vertex::new([-1.0, 1.0]),
                                          Vertex::new([-1.0, 1.0]),
                                          Vertex::new([1.0, -1.0]),
                                          Vertex::new([1.0, 1.0])];

        let attributes = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .finish();

        let layout = Vertex::layout();
        let state = graphics::RenderState::default();

        let vbo = app.graphics
            .create_vertex_buffer(&layout,
                                  graphics::BufferHint::Static,
                                  48,
                                  Some(Vertex::as_bytes(&quad_vertices[..])))
            .unwrap();
        let view = app.graphics.create_view(None).unwrap();
        let pipeline = app.graphics
            .create_pipeline(include_str!("resources/shaders/texture.vs"),
                             include_str!("resources/shaders/texture.fs"),
                             &state,
                             &attributes)
            .unwrap();

        let texture: TexturePtr = app.resources.load("texture.png").unwrap();

        Ok(Window {
               view: view,
               pso: pipeline,
               vbo: vbo,
               texture: texture,
           })
    }
}

impl ApplicationInstance for Window {
    fn on_update(&mut self, app: &mut Application) -> errors::Result<()> {
        let mut texture = self.texture.write().unwrap();
        texture.update_video_object(&mut app.graphics)?;

        {
            let len = self.vbo.object.read().unwrap().len();
            let mut task = app.graphics.make();
            task.with_order(0)
                .with_view(*self.view)
                .with_pipeline(*self.pso)
                .with_data(*self.vbo, None)
                .with_texture("renderedTexture", texture.video_object().unwrap())
                .submit(graphics::Primitive::Triangles, 0, len)?;
        }

        Ok(())
    }
}

fn main() {
    utils::compile();

    let mut settings = Settings::default();
    settings.window.width = 232;
    settings.window.height = 217;

    let manifest = "examples/compiled-resources/manifest";
    let mut app = Application::new_with(settings).unwrap();
    app.resources.load_manifest(manifest).unwrap();

    let mut window = Window::new(&mut app).unwrap();
    app.run(&mut window).unwrap();
}