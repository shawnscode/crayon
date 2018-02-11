use std::sync::Arc;
use application::Context;
use ecs::{Arena, Entity, Fetch, System, View, World};

use graphics;
use resource;
use math;

use scene::{Camera, Element, Node, Transform};
use scene::assets::factory;
use scene::errors::*;

pub enum SceneDrawOrder {
    Shadow = 0,
    // Camera,
}

/// A shadow mapping builder.
pub struct RenderShadow {
    video: Arc<graphics::GraphicsSystemShared>,

    depth_shadow_texture: graphics::RenderTextureHandle,
    depth_surface: graphics::SurfaceHandle,
    depth_shader: graphics::ShaderHandle,
    draw_shader: graphics::ShaderHandle,
}

impl RenderShadow {
    /// Craetes a new `RenderShadow`.
    pub fn new(ctx: &Context) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>().clone();

        let render_depth_buffer = {
            let mut setup = graphics::RenderTextureSetup::default();
            setup.format = graphics::RenderTextureFormat::Depth16;
            setup.dimensions = (640, 480);
            video.create_render_texture(setup)?
        };

        let surface = {
            let mut setup = graphics::SurfaceSetup::default();
            setup.set_attachments(&[], render_depth_buffer)?;
            setup.set_clear(None, 1.0, None);
            setup.set_order(SceneDrawOrder::Shadow as u64);
            video.create_surface(setup)?
        };

        let shader = {
            let attributes = graphics::AttributeLayoutBuilder::new()
                .with(graphics::Attribute::Position, 3)
                .finish();

            let mut setup = graphics::ShaderSetup::default();
            setup.render_state.depth_write = true;
            setup.render_state.depth_test = graphics::Comparison::Less;
            setup.render_state.cull_face = graphics::CullFace::Back;
            setup.layout = attributes;
            setup.vs = include_str!("../assets/shaders/shadow.vs").to_owned();
            setup.fs = include_str!("../assets/shaders/shadow.fs").to_owned();

            let tt = graphics::UniformVariableType::Matrix4f;
            setup.uniform_variables.insert("u_MVPMatrix".into(), tt);
            video.create_shader(resource::Location::unique(""), setup)?
        };

        let draw_shader = {
            let attributes = graphics::AttributeLayoutBuilder::new()
                .with(graphics::Attribute::Position, 3)
                .finish();

            let mut setup = graphics::ShaderSetup::default();
            setup.layout = attributes;
            setup.vs = include_str!("../assets/shaders/shadow_texture.vs").to_owned();
            setup.fs = include_str!("../assets/shaders/shadow_texture.fs").to_owned();

            let tt = graphics::UniformVariableType::RenderTexture;
            setup.uniform_variables.insert("u_ShadowTexture".into(), tt);
            video.create_shader(resource::Location::unique(""), setup)?
        };

        Ok(RenderShadow {
            video: video,

            depth_shadow_texture: render_depth_buffer,
            depth_surface: surface,
            depth_shader: shader,
            draw_shader: draw_shader,
        })
    }

    /// Gets the handle of depth buffer.
    pub fn texture(&self) -> graphics::RenderTextureHandle {
        self.depth_shadow_texture
    }

    /// Builds the depth buffer of shadow mapping technique, and returns the light
    /// space transformation matrix.
    pub fn build_shadow_texture(
        &self,
        world: &World,
        caster: Entity,
    ) -> Result<math::Matrix4<f32>> {
        GenerateRenderShadow {
            shadow: self,
            caster: caster,
        }.run_at(world)
    }

    /// Draw the underlying depth buffer into the `surface`.
    pub fn draw(&self, surface: graphics::SurfaceHandle) -> Result<()> {
        let mesh = factory::mesh::quad(&self.video)?;
        let mut dc = graphics::DrawCall::new(self.draw_shader, mesh);
        dc.set_uniform_variable("u_ShadowTexture", self.depth_shadow_texture);
        let sdc = dc.build_sub_mesh(0)?;

        self.video.submit(surface, 0u64, sdc)?;
        Ok(())
    }
}

impl Drop for RenderShadow {
    fn drop(&mut self) {
        self.video.delete_render_texture(self.depth_shadow_texture);
        self.video.delete_surface(self.depth_surface);
        self.video.delete_shader(self.depth_shader);
        self.video.delete_shader(self.draw_shader);
    }
}

struct GenerateRenderShadow<'a> {
    shadow: &'a RenderShadow,
    caster: Entity,
}

impl<'a, 'b> System<'a> for GenerateRenderShadow<'b> {
    type ViewWith = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, Element>);
    type Result = Result<math::Matrix4<f32>>;

    fn run(&self, view: View, data: Self::ViewWith) -> Self::Result {
        let v = Transform::world_view_matrix(&data.0, &data.1, self.caster)?;
        let p = Camera::ortho_matrix(-256.0, 256.0, -256.0, 256.0, 0.1, 1000.0);
        let vp = p * v;

        unsafe {
            for handle in view {
                if let Element::Mesh(mesh) = *data.2.get_unchecked(handle) {
                    let point = Transform::world_position(&data.0, &data.1, handle).unwrap();
                    let mut csp = v * math::Vector4::new(point.x, point.y, point.z, 1.0);
                    csp /= csp.w;

                    if csp.z <= 0.0 {
                        continue;
                    }

                    let m = Transform::world_matrix(&data.0, &data.1, handle)?;
                    let mvp = vp * m;

                    let mut dc = graphics::DrawCall::new(self.shadow.depth_shader, mesh.mesh);
                    dc.set_uniform_variable("u_MVPMatrix", mvp);
                    let sdc = dc.build(mesh.index)?;

                    self.shadow
                        .video
                        .submit(self.shadow.depth_surface, 0u64, sdc)?;
                }
            }
        }

        Ok(vp)
    }
}
