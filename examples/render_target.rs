#[macro_use]
extern crate crayon;

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
    texture: graphics::TextureRef,

    view_pass_2: graphics::ViewStateRef,
    pso_pass_2: graphics::PipelineStateRef,
    vbo_pass_2: graphics::VertexBufferRef,

    time: f32,
}

impl Window {
    pub fn new(app: &mut Application) -> errors::Result<Self> {
        let vertices: [Vertex; 3] = [Vertex::new([0.0, 0.5]),
                                     Vertex::new([0.5, -0.5]),
                                     Vertex::new([-0.5, -0.5])];

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

        //
        let vbo_fb = app.graphics
            .create_vertex_buffer(&layout,
                                  graphics::ResourceHint::Static,
                                  24,
                                  Some(Vertex::as_bytes(&vertices[..])))?;

        let state = graphics::RenderState::default();
        let rendered_texture =
            app.graphics
                .create_render_texture(graphics::RenderTextureFormat::RGBA8, 568, 320)?;

        let fbo = app.graphics.create_framebuffer()?;
        {
            let mut item = fbo.object.write().unwrap();
            item.update_texture_attachment(&rendered_texture, Some(0))?;
            item.update_clear(Some(Color::gray()), None, None);
        }

        let view_fb = app.graphics.create_view(Some(fbo))?;
        let pipeline_fb = app.graphics
            .create_pipeline(include_str!("resources/shaders/render_target_p1.vs"),
                             include_str!("resources/shaders/render_target_p1.fs"),
                             &state,
                             &attributes)?;

        let vbo = app.graphics
            .create_vertex_buffer(&layout,
                                  graphics::ResourceHint::Static,
                                  48,
                                  Some(Vertex::as_bytes(&quad_vertices[..])))?;
        let view = app.graphics.create_view(None)?;
        let pipeline = app.graphics
            .create_pipeline(include_str!("resources/shaders/render_target_p2.vs"),
                             include_str!("resources/shaders/render_target_p2.fs"),
                             &state,
                             &attributes)?;

        Ok(Window {
               view: view_fb,
               pso: pipeline_fb,
               vbo: vbo_fb,
               texture: rendered_texture,

               view_pass_2: view,
               pso_pass_2: pipeline,
               vbo_pass_2: vbo,

               time: 0.0,
           })
    }
}

impl ApplicationInstance for Window {
    fn on_update(&mut self, app: &mut Application) -> errors::Result<()> {
        let mut uniforms = vec![];
        let mut textures = vec![];
        app.graphics
            .draw(0,
                  *self.view,
                  *self.pso,
                  textures.as_slice(),
                  uniforms.as_slice(),
                  *self.vbo,
                  None,
                  graphics::Primitive::Triangles,
                  0,
                  self.vbo.object.read().unwrap().len())
            .unwrap();

        textures.push(("renderedTexture", *self.texture));
        uniforms.push(("time", graphics::UniformVariable::F32(self.time)));

        app.graphics
            .draw(0,
                  *self.view_pass_2,
                  *self.pso_pass_2,
                  textures.as_slice(),
                  uniforms.as_slice(),
                  *self.vbo_pass_2,
                  None,
                  graphics::Primitive::Triangles,
                  0,
                  self.vbo_pass_2.object.read().unwrap().len())
            .unwrap();

        self.time += 0.05;
        Ok(())
    }
}

fn main() {
    let mut settings = crayon::core::settings::Settings::default();
    settings.window.width = 568;
    settings.window.height = 320;

    let mut app = Application::new_with(settings).unwrap();
    let mut window = Window::new(&mut app).unwrap();
    app.run(&mut window).unwrap();
}