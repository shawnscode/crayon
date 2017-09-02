use core::application;
use ecs;
use graphics;

use super::errors::*;
use super::{Transform, Mesh, Renderable, RenderCamera};

pub struct MeshRenderer {}

impl MeshRenderer {
    pub fn new(_: &mut application::Application) -> Result<Self> {
        Ok(MeshRenderer {})
    }

    pub fn draw(&mut self,
                mut application: &mut application::Application,
                world: &ecs::World,
                camera: &RenderCamera)
                -> Result<()> {

        let (view, mut arenas) = world.view_with_2::<Transform, Mesh>();

        for v in view {
            self.submit(&mut application, &camera, &mut arenas, v).ok();
        }

        Ok(())
    }

    fn submit(&self,
              application: &mut application::Application,
              camera: &RenderCamera,
              arenas: &mut (ecs::ArenaGetter<Transform>, ecs::ArenaGetter<Mesh>),
              v: ecs::Entity)
              -> Result<()> {
        use graphics::UniformVariable as UV;
        use graphics::UniformVariableType as UVT;

        let mesh = arenas.1.get(*v).unwrap();
        if !mesh.visible() || mesh.material().is_none() {
            return Ok(());
        }

        let position = Transform::world_position(&arenas.0, v)?;
        let csp = camera.transform(&position);

        if !camera.is_inside(&csp) {
            return Ok(());
        }

        let mat = mesh.material().unwrap();
        let mat = mat.write().unwrap();
        let mut uniforms = Vec::new();
        let mut textures = Vec::new();
        mat.build_uniform_variables(&mut application.graphics, &mut textures, &mut uniforms)?;

        // Assemble uniform variables with build-in uniforms.
        if mat.has_uniform_variable("bi_ViewMatrix", UVT::Matrix4f) {
            uniforms.push(("bi_ViewMatrix", UV::Matrix4f(*camera.view.as_ref(), true)));
        }

        if mat.has_uniform_variable("bi_ProjectionMatrix", UVT::Matrix4f) {
            uniforms.push(("bi_ProjectionMatrix", UV::Matrix4f(*camera.view.as_ref(), true)));
        }

        // Get pipeline state object from shader.
        let pso = {
            let mut shader = mat.shader().write().unwrap();
            shader.update_video_object(&mut application.graphics)?;
            shader.video_object().unwrap()
        };

        // Get primitive buffer objects from mesh.
        let (vbo, ibo, len) = {
            let mut primitive = mesh.primitive().write().unwrap();
            primitive.update_video_object(&mut application.graphics)?;

            let (vbo, ibo) = primitive.video_object().unwrap();
            let len = if ibo.is_none() {
                primitive.vlen()
            } else {
                primitive.ilen().unwrap()
            };

            (vbo, ibo, len as u32)
        };

        // Submit draw call with packed order.
        let order = {
            let shader = mat.shader().read().unwrap();
            DrawOrder {
                tranlucent: shader.render_state().color_blend.is_some(),
                zorder: (csp.z.min(camera.clip.0).max(camera.clip.1) * 1000f32) as u32,
                pso: pso,
            }
        };

        application
            .graphics
            .draw(order.into(),
                  camera.vso,
                  pso,
                  &textures,
                  &uniforms,
                  vbo,
                  ibo,
                  graphics::Primitive::Triangles,
                  0,
                  len)?;

        Ok(())
    }
}

struct DrawOrder {
    pub tranlucent: bool,
    pub zorder: u32,
    pub pso: graphics::PipelineStateHandle,
}

impl Into<u64> for DrawOrder {
    fn into(self) -> u64 {
        let prefix = if self.tranlucent {
            (!self.zorder)
        } else {
            self.zorder
        };

        let suffix = self.pso.index();
        ((prefix as u64) << 32) | (suffix as u64)
    }
}