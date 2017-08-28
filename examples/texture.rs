#[macro_use]
extern crate crayon;

use crayon::graphics::*;
use crayon::resource;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    view: ViewStateRef,
    pso: PipelineStateRef,
    vbo: VertexBufferRef,
    texture: resource::TextureItem,
}

fn main() {
    let mut window = None;
    let mut settings = crayon::core::settings::Settings::default();
    settings.window.width = 232;
    settings.window.height = 217;

    crayon::Application::new_with(settings)
        .unwrap()
        .perform(|app| {
            let quad_vertices: [Vertex; 6] = [Vertex::new([-1.0, -1.0]),
                                              Vertex::new([1.0, -1.0]),
                                              Vertex::new([-1.0, 1.0]),
                                              Vertex::new([-1.0, 1.0]),
                                              Vertex::new([1.0, -1.0]),
                                              Vertex::new([1.0, 1.0])];

            let attributes = AttributeLayoutBuilder::new()
                .with(VertexAttribute::Position, 2)
                .finish();

            let layout = Vertex::layout();
            let state = RenderState::default();

            let vbo = app.graphics
                .create_vertex_buffer(&layout,
                                      ResourceHint::Static,
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

            app.resources
                .load_manifest("examples/compiled-resources/manifest")
                .unwrap();

            let texture: resource::TextureItem = app.resources.load("texture.png").unwrap();

            texture
                .write()
                .unwrap()
                .update_video_object(&mut app.graphics)
                .unwrap();

            window = Some(Window {
                              view: view,
                              pso: pipeline,
                              vbo: vbo,
                              texture: texture,
                          });
        })
        .run(move |app| {
            if let Some(ref mut window) = window {
                let uniforms = vec![];
                let mut textures = vec![];

                textures.push(("renderedTexture",
                               window.texture.read().unwrap().video_object().unwrap()));
                app.graphics
                    .draw(0,
                          *window.view,
                          *window.pso,
                          textures.as_slice(),
                          uniforms.as_slice(),
                          *window.vbo,
                          None,
                          Primitive::Triangles,
                          0,
                          window.vbo.object.read().unwrap().len())
                    .unwrap();
            }
            return true;
        })
        .perform(|_| println!("hello world."));
}