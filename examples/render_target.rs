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
    pub fn new(engine: &mut Engine) -> errors::Result<Self> {
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
        let vbo_fb = engine
            .graphics
            .create_vertex_buffer(&layout,
                                  graphics::BufferHint::Static,
                                  (vertices.len() * layout.stride() as usize) as u32,
                                  Some(Vertex::as_bytes(&vertices[..])))?;

        let state = graphics::RenderState::default();
        let rendered_texture =
            engine
                .graphics
                .create_render_texture(graphics::RenderTextureFormat::RGBA8, 568, 320)?;

        let fbo = engine.graphics.create_framebuffer()?;
        {
            let mut item = fbo.object.write().unwrap();
            item.update_texture_attachment(&rendered_texture, Some(0))?;
        }

        let view_fb = engine.graphics.create_view(Some(fbo))?;
        {
            let mut item = view_fb.object.write().unwrap();
            item.update_clear(Some(Color::gray()), None, None);
        }

        let pipeline_fb = engine
            .graphics
            .create_pipeline(include_str!("resources/shaders/render_target_p1.vs"),
                             include_str!("resources/shaders/render_target_p1.fs"),
                             &state,
                             &attributes)?;

        let vbo = engine
            .graphics
            .create_vertex_buffer(&layout,
                                  graphics::BufferHint::Static,
                                  (quad_vertices.len() * layout.stride() as usize) as u32,
                                  Some(Vertex::as_bytes(&quad_vertices[..])))?;
        let view = engine.graphics.create_view(None)?;
        let pipeline = engine
            .graphics
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

impl Application for Window {
    fn on_update(&mut self, app: &mut Engine) -> errors::Result<()> {
        {
            let len = self.vbo.object.read().unwrap().len();
            let mut task = app.graphics.make();
            task.with_order(0)
                .with_view(*self.view)
                .with_pipeline(*self.pso)
                .with_data(*self.vbo, None)
                .submit(graphics::Primitive::Triangles, 0, len)?;
        }

        {
            let len = self.vbo_pass_2.object.read().unwrap().len();
            let mut task = app.graphics.make();
            task.with_order(1)
                .with_view(*self.view_pass_2)
                .with_pipeline(*self.pso_pass_2)
                .with_data(*self.vbo_pass_2, None)
                .with_uniform_variable("time", self.time.into())
                .with_texture("renderedTexture", *self.texture)
                .submit(graphics::Primitive::Triangles, 0, len)?;
        }

        self.time += 0.05;
        Ok(())
    }
}

fn main() {
    let mut settings = Settings::default();
    settings.window.width = 568;
    settings.window.height = 320;

    let mut engine = Engine::new_with(settings).unwrap();
    let mut window = Window::new(&mut engine).unwrap();
    engine.run(&mut window).unwrap();
}