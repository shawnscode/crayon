use application;
use ecs;
use graphics;
use scene::errors::*;

use std::sync::Arc;

impl_vertex!{
    PbrVertex {
        position => [Position; Float; 2; false],
        texcoord => [Texcoord0; Float; 2; false],
        normal => [Normal; Float; 4; false],
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PbrMesh {
    pub mesh: graphics::MeshHandle,
    pub index: graphics::MeshIndex,
    pub material: PbrMaterial,
}

#[derive(Debug, Copy, Clone)]
pub struct PbrMaterial {}

pub struct PbrMeshRenderer {
    video: Arc<graphics::GraphicsSystemShared>,
    shader: graphics::ShaderHandle,
}

impl PbrMeshRenderer {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>();

        let mut setup = graphics::ShaderSetup::default();
        setup.layout = PbrVertex::attributes();
        setup.render_state.color_blend = Some((
            graphics::Equation::Add,
            graphics::BlendFactor::Value(graphics::BlendValue::SourceAlpha),
            graphics::BlendFactor::OneMinusValue(graphics::BlendValue::SourceAlpha),
        ));
        setup.vs = include_str!("../assets/pbr.vs").to_owned();
        setup.fs = include_str!("../assets/pbr.fs").to_owned();

        setup.uniform_variables.push("u_MVPMatrix".into());
        setup.uniform_variables.push("u_ModelViewMatrix".into());
        setup.uniform_variables.push("u_NormalMatrix".into());

        let shader = video.create_shader(setup)?;

        Ok(PbrMeshRenderer {
            video: video.clone(),
            shader: shader,
        })
    }

    pub fn draw(world: &ecs::World, surface: graphics::SurfaceHandle) -> Result<()> {
        Ok(())
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
