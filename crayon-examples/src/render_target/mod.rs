use crayon::prelude::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Pass {
    view: graphics::SurfaceHandle,
    shader: graphics::ShaderHandle,
    mesh: graphics::VertexBufferHandle,
}

struct Window {
    _label: graphics::RAIIGuard,
    pass: Pass,
    post_effect: Pass,
    texture: graphics::TextureHandle,
    time: f32,
}

impl Window {
    pub fn new(engine: &mut Engine) -> errors::Result<Self> {
        let ctx = engine.context();
        let video = ctx.shared::<GraphicsSystem>().clone();
        let mut label = graphics::RAIIGuard::new(video);

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

        //
        let (pass, rendered_texture) = {
            // Create vertex buffer object.
            let mut setup = graphics::VertexBufferSetup::default();
            setup.num = vertices.len() as u32;
            setup.layout = Vertex::layout();
            let vbo = label
                .create_vertex_buffer(setup, Some(Vertex::as_bytes(&vertices[..])))?;

            // Create render texture for post effect.
            let mut setup = graphics::RenderTextureSetup::default();
            setup.format = graphics::RenderTextureFormat::RGBA8;
            setup.dimensions = (568, 320);
            let rendered_texture = label.create_render_texture(setup)?;

            // Create custom frame buffer.
            let mut setup = graphics::FrameBufferSetup::default();
            setup.set_attachment(rendered_texture, 0)?;
            let fbo = label.create_framebuffer(setup)?;

            // Create the view state for pass 1.
            let mut setup = graphics::SurfaceSetup::default();
            setup.set_order(0);
            setup.set_framebuffer(fbo);
            setup.set_clear(Color::gray(), None, None);
            let view = label.create_surface(setup)?;

            // Create shader state.
            let mut setup = graphics::ShaderSetup::default();
            setup.layout = attributes;
            setup.vs = include_str!("../../resources/render_target_p1.vs").to_owned();
            setup.fs = include_str!("../../resources/render_target_p1.fs").to_owned();
            let shader = label.create_shader(setup)?;

            (Pass {
                 view: view,
                 shader: shader,
                 mesh: vbo,
             },
             rendered_texture)
        };

        let post_effect = {
            let mut setup = graphics::VertexBufferSetup::default();
            setup.num = quad_vertices.len() as u32;
            setup.layout = Vertex::layout();
            let vbo = label
                .create_vertex_buffer(setup, Some(Vertex::as_bytes(&quad_vertices[..])))?;

            let mut setup = graphics::SurfaceSetup::default();
            setup.set_order(1);
            let view = label.create_surface(setup)?;

            let mut setup = graphics::ShaderSetup::default();
            setup.layout = attributes;
            setup.vs = include_str!("../../resources/render_target_p2.vs").to_owned();
            setup.fs = include_str!("../../resources/render_target_p2.fs").to_owned();
            setup.uniform_variables.push("renderedTexture".into());
            setup.uniform_variables.push("time".into());
            let shader = label.create_shader(setup)?;

            Pass {
                view: view,
                shader: shader,
                mesh: vbo,
            }
        };

        Ok(Window {
               _label: label,

               pass: pass,
               post_effect: post_effect,
               texture: rendered_texture,

               time: 0.0,
           })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> errors::Result<()> {
        let video = ctx.shared::<GraphicsSystem>();

        {
            let mut dc = graphics::DrawCall::new(self.pass.shader);
            dc.set_mesh(self.pass.mesh, None);
            let cmd = dc.build(graphics::Primitive::Triangles, 0, 3)?;
            video.submit(self.pass.view, 0, cmd)?;
        }

        {
            let mut dc = graphics::DrawCall::new(self.post_effect.shader);
            dc.set_mesh(self.post_effect.mesh, None);
            dc.set_uniform_variable("renderedTexture", self.texture);
            dc.set_uniform_variable("time", self.time);
            let cmd = dc.build(graphics::Primitive::Triangles, 0, 6)?;
            video.submit(self.post_effect.view, 1, cmd)?;
        }

        self.time += 0.05;
        Ok(())
    }
}

pub fn main(title: String, _: &[String]) {
    let mut settings = Settings::default();
    settings.window.width = 568;
    settings.window.height = 320;
    settings.window.title = title;

    let mut engine = Engine::new_with(settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}