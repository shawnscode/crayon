use ecs;
use ecs::Arena;
use math;
use math::{Matrix, SquareMatrix};
use graphics;
use utils::Handle;
use utils;

use scene::{material, node, transform};
use scene::scene::SceneNode;

#[derive(Debug, Copy, Clone)]
pub struct MeshRenderer {
    pub mesh: graphics::MeshHandle,
    pub index: graphics::MeshIndex,
    pub material: Handle,
}

pub(crate) struct RenderTask<'a> {
    pub(crate) video: &'a graphics::GraphicsSystemShared,
    pub(crate) materials: &'a utils::HandleObjectPool<material::Material>,
    pub(crate) fallback: &'a material::Material,
    pub(crate) surface: graphics::SurfaceHandle,
    pub(crate) view_matrix: math::Matrix4<f32>,
    pub(crate) projection_matrix: math::Matrix4<f32>,
}

type RenderData<'a> = (
    ecs::Fetch<'a, node::Node>,
    ecs::Fetch<'a, transform::Transform>,
    ecs::Fetch<'a, SceneNode>,
);

impl<'a, 'b> ecs::System<'a> for RenderTask<'b> {
    type ViewWith = RenderData<'a>;

    fn run(&self, view: ecs::View, data: Self::ViewWith) {
        let vp = self.projection_matrix * self.view_matrix;
        unsafe {
            for v in view {
                if let &SceneNode::Mesh(mesh) = data.2.get_unchecked(v) {
                    let mut mat = self.materials.get(mesh.material).unwrap_or(self.fallback);
                    if !self.video.is_shader_alive(mat.shader()) {
                        mat = self.fallback;
                    }

                    // Generate packed draw order.
                    let p = transform::Transform::world_position(&data.0, &data.1, v).unwrap();
                    let mut csp = self.view_matrix * math::Vector4::new(p.x, p.y, p.z, 1.0);
                    csp /= csp.w;

                    let order = DrawOrder {
                        tranlucent: mat.render_state().color_blend.is_some(),
                        zorder: (csp.z * 1000.0) as u32,
                        shader: mat.shader(),
                    };

                    // Generate draw call and fill it with build-in uniforms.
                    let mut dc = graphics::DrawCall::new(mat.shader(), mesh.mesh);
                    let m = transform::Transform::world_matrix(&data.0, &data.1, v).unwrap();
                    let mv = self.view_matrix * m;

                    for (k, v) in &mat.variables {
                        dc.set_uniform_variable(*k, *v);
                    }

                    if mat.has_uniform_variable("u_ModelMatrix") {
                        dc.set_uniform_variable("u_ModelMatrix", m);
                    }

                    if mat.has_uniform_variable("u_ModelViewMatrix") {
                        dc.set_uniform_variable("u_ModelViewMatrix", mv);
                    }

                    if mat.has_uniform_variable("u_MVPMatrix") {
                        dc.set_uniform_variable("u_MVPMatrix", vp * m);
                    }

                    if mat.has_uniform_variable("u_NormalMatrix") {
                        let n = if let Some(invert) = mv.invert() {
                            invert.transpose()
                        } else {
                            mv
                        };

                        dc.set_uniform_variable("u_NormalMatrix", n);
                    }

                    let sdc = dc.build(mesh.index).unwrap();

                    // Submit.
                    self.video.submit(self.surface, order, sdc).unwrap();
                }
            }
        }
    }
}

struct DrawOrder {
    tranlucent: bool,
    zorder: u32,
    shader: graphics::ShaderHandle,
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
