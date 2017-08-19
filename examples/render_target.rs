#[macro_use]
extern crate crayon;

use crayon::graphics::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    view: ViewStateRef,
    pso: PipelineStateRef,
    vbo: VertexBufferRef,
    texture: TextureRef,

    view_pass_2: ViewStateRef,
    pso_pass_2: PipelineStateRef,
    vbo_pass_2: VertexBufferRef,

    time: f32,
}

fn main() {
    let mut window = None;

    let mut settings = crayon::core::settings::Settings::default();
    settings.window.width = 568;
    settings.window.height = 320;

    crayon::Application::new_with(settings)
        .unwrap()
        .perform(|app| {
            let vertices: [Vertex; 3] = [Vertex::new([0.0, 0.5]),
                                         Vertex::new([0.5, -0.5]),
                                         Vertex::new([-0.5, -0.5])];

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

            //
            let vbo_fb = app.graphics
                .create_vertex_buffer(&layout,
                                      ResourceHint::Static,
                                      24,
                                      Some(Vertex::as_bytes(&vertices[..])))
                .unwrap();

            let state = RenderState::default();
            let rendered_texture = app.graphics
                .create_render_texture(RenderTextureFormat::RGBA8, 568, 320)
                .unwrap();

            let fbo = app.graphics.create_framebuffer().unwrap();
            {
                let mut item = fbo.object.write().unwrap();
                item.update_texture_attachment(&rendered_texture, Some(0))
                    .unwrap();
                item.update_clear(Some(Color::gray()), None, None);
            }

            let view_fb = app.graphics.create_view(Some(&fbo)).unwrap();
            let pipeline_fb = app.graphics
                .create_pipeline(include_str!("resources/shaders/render_target_p1.vs"),
                                 include_str!("resources/shaders/render_target_p1.fs"),
                                 &state,
                                 &attributes)
                .unwrap();

            let vbo = app.graphics
                .create_vertex_buffer(&layout,
                                      ResourceHint::Static,
                                      48,
                                      Some(Vertex::as_bytes(&quad_vertices[..])))
                .unwrap();
            let view = app.graphics.create_view(None).unwrap();
            let pipeline = app.graphics
                .create_pipeline(include_str!("resources/shaders/render_target_p2.vs"),
                                 include_str!("resources/shaders/render_target_p2.fs"),
                                 &state,
                                 &attributes)
                .unwrap();

            window = Some(Window {
                              view: view_fb,
                              pso: pipeline_fb,
                              vbo: vbo_fb,
                              texture: rendered_texture,

                              view_pass_2: view,
                              pso_pass_2: pipeline,
                              vbo_pass_2: vbo,

                              time: 0.0,
                          });
        })
        .run(move |app| {
            if let Some(ref mut window) = window {
                let mut uniforms = vec![];
                let mut textures = vec![];
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

                textures.push(("renderedTexture", *window.texture));
                uniforms.push(("time", UniformVariable::F32(window.time)));
                app.graphics
                    .draw(0,
                          *window.view_pass_2,
                          *window.pso_pass_2,
                          textures.as_slice(),
                          uniforms.as_slice(),
                          *window.vbo_pass_2,
                          None,
                          Primitive::Triangles,
                          0,
                          window.vbo_pass_2.object.read().unwrap().len())
                    .unwrap();

                window.time += 0.05;
            }
            return true;
        })
        .perform(|_| println!("hello world."));
}