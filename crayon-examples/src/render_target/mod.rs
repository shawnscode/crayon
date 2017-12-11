use crayon::prelude::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Pass {
    view: graphics::ViewStateHandle,
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
        let ctx = engine.context().read().unwrap();
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
            setup.num = vertices.len();
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
            setup.set_texture_attachment(rendered_texture, Some(0))?;
            let fbo = label.create_framebuffer(setup)?;

            // Create the view state for pass 1.
            let mut setup = graphics::ViewStateSetup::default();
            setup.framebuffer = Some(fbo);
            setup.clear_color = Some(Color::gray());
            let view = label.create_view(setup)?;

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
            setup.num = quad_vertices.len();
            setup.layout = Vertex::layout();
            let vbo = label
                .create_vertex_buffer(setup, Some(Vertex::as_bytes(&quad_vertices[..])))?;

            let setup = graphics::ViewStateSetup::default();
            let view = label.create_view(setup)?;

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
            let mut dc = DrawCall::new(self.pass.view, self.pass.shader);
            dc.set_mesh(self.pass.mesh, None);
            dc.submit(&video, graphics::Primitive::Triangles, 0, 3)?;
        }

        {
            let mut dc = DrawCall::new(self.post_effect.view, self.post_effect.shader);
            dc.set_mesh(self.post_effect.mesh, None);
            dc.set_uniform_variable("renderedTexture", self.texture);
            dc.set_uniform_variable("time", self.time);
            dc.submit(&video, graphics::Primitive::Triangles, 0, 6)?;
        }

        self.time += 0.05;
        Ok(())
    }
}

pub fn main(_: &[String]) {
    let mut settings = Settings::default();
    settings.window.width = 568;
    settings.window.height = 320;

    let mut engine = Engine::new_with(settings).unwrap();
    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}