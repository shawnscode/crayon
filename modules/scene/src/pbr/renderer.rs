use crayon::{application, graphics, ecs};

impl_vertex!{
    CanvasVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        color => [Color0; UByte; 4; true],
    }
}

pub struct Renderer {
    video: Arc<graphics::GraphicsSystemShared>,
    shader: graphics::ShaderHandle,
}

impl Renderer {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>();

        let layout = graphics::AttributeLayoutBuilder::new()
            .with(graphics::VertexAttribute::Position, 2)
            .with(graphics::VertexAttribute::Texcoord0, 2)
            .with(graphics::VertexAttribute::Color0, 4)
            .finish();

        let mut setup = graphics::ShaderSetup::default();
        setup.layout = layout;
        setup.render_state.color_blend =
            Some((graphics::Equation::Add,
                  graphics::BlendFactor::Value(graphics::BlendValue::SourceAlpha),
                  graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha)));
        setup.vs = include_str!("../../assets/pbr.vs").to_owned();
        setup.fs = include_str!("../../assets/pbr.fs").to_owned();

        // setup.uniform_variables.push("u_ProjectionMatrix".into());
        // setup.uniform_variables.push("u_ViewMatrix".into());
        // setup.uniform_variables.push("u_ModelMatrix".into());

        let shader = video.create_shader(setup)?;

        Renderer {
            video: video.clone(),
            shader: shader,
        }
    }

    pub fn draw(world: &ecs::World, surface: graphics::SurfaceHandle) -> Result<()> {
        let render 
    }
}

struct RenderSystem {
    video: Arc<graphics::GraphicsSystemShared>,
    shader: graphics::ShaderHandle,
    surface: graphics::SurfaceHandle,
}



struct DrawOrder {
    pub tranlucent: bool,
    pub zorder: u32,
    pub shader: graphics::ShaderHandle,
}

impl Into<u64> for DrawOrder {
    fn into(self) -> u64 {
        let prefix = if self.tranlucent {
            (!self.zorder)
        } else {
            self.zorder
        };

        let suffix = self.shader.index();
        ((prefix as u64) << 32) | (suffix as u64)
    }
}