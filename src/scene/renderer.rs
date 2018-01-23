use ecs::{Arena, Fetch, System, View};
use math;
use math::{Matrix, SquareMatrix};
use graphics::{DrawCall, GraphicsSystemShared, MeshHandle, MeshIndex, ShaderHandle, SurfaceHandle};
use utils::HandleObjectPool;

use scene::{LightSource, Node, Transform};
use scene::material::{Material, MaterialHandle};
use scene::scene::SceneNode;

#[derive(Debug, Copy, Clone)]
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub index: MeshIndex,
    pub material: MaterialHandle,
}

type SceneViewData<'a> = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, SceneNode>);

#[derive(Debug, Clone)]
pub(crate) struct RenderDataDirLight {
    /// Direction in eye space.
    pub forward: math::Vector3<f32>,
    pub dir: math::Vector3<f32>,
    pub dir_field: String,
    pub color: math::Vector3<f32>,
    pub color_field: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderDataPointLight {
    /// Position in eye space.
    pub position: math::Vector3<f32>,
    pub position_field: String,
    pub color: math::Vector3<f32>,
    pub color_field: String,
    pub attenuation: math::Vector3<f32>,
    pub attenuation_field: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderData {
    pub dir: Option<RenderDataDirLight>,
    pub points: Vec<RenderDataPointLight>,
}

pub(crate) struct RenderTask<'a> {
    pub video: &'a GraphicsSystemShared,
    pub materials: &'a HandleObjectPool<Material>,
    pub fallback: &'a Material,
    pub surface: SurfaceHandle,
    pub view_matrix: math::Matrix4<f32>,
    pub projection_matrix: math::Matrix4<f32>,
    pub data: RenderData,
}

impl<'a, 'b> System<'a> for RenderTask<'b> {
    type ViewWith = SceneViewData<'a>;

    fn run(&self, view: View, data: Self::ViewWith) {
        let vp = self.projection_matrix * self.view_matrix;
        unsafe {
            for v in view {
                if let &SceneNode::Mesh(mesh) = data.2.get_unchecked(v) {
                    let mut mat = self.materials.get(mesh.material).unwrap_or(self.fallback);
                    if !self.video.is_shader_alive(mat.shader()) {
                        mat = self.fallback;
                    }

                    // Generate packed draw order.
                    let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                    let mut csp = self.view_matrix * math::Vector4::new(p.x, p.y, p.z, 1.0);
                    csp /= csp.w;

                    let order = DrawOrder {
                        tranlucent: mat.render_state().color_blend.is_some(),
                        zorder: (csp.z * 1000.0) as u32,
                        shader: mat.shader(),
                    };

                    // Generate draw call and fill it with build-in uniforms.
                    let mut dc = DrawCall::new(mat.shader(), mesh.mesh);
                    let m = Transform::world_matrix(&data.0, &data.1, v).unwrap();
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

                    if let &Some(ref dir) = &self.data.dir {
                        if mat.has_uniform_variable(&dir.dir_field) {
                            dc.set_uniform_variable(&dir.dir_field, dir.dir);
                        }

                        if mat.has_uniform_variable(&dir.color_field) {
                            dc.set_uniform_variable(&dir.color_field, dir.color);
                        }
                    }

                    for v in &self.data.points {
                        if mat.has_uniform_variable(&v.position_field) {
                            dc.set_uniform_variable(&v.position_field, v.position);
                        }

                        if mat.has_uniform_variable(&v.color_field) {
                            dc.set_uniform_variable(&v.color_field, v.color);
                        }

                        if mat.has_uniform_variable(&v.attenuation_field) {
                            dc.set_uniform_variable(&v.attenuation_field, v.attenuation);
                        }
                    }

                    let sdc = dc.build(mesh.index).unwrap();

                    // Submit.
                    self.video.submit(self.surface, order, sdc).unwrap();
                }
            }
        }
    }
}

pub(crate) struct RenderDataCollectTask {
    pub data: RenderData,
    pub view_matrix: math::Matrix4<f32>,
}

impl RenderDataCollectTask {
    pub fn new(view: math::Matrix4<f32>) -> Self {
        RenderDataCollectTask {
            view_matrix: view,
            data: RenderData {
                dir: None,
                points: Vec::new(),
            },
        }
    }
}

impl<'a> System<'a> for RenderDataCollectTask {
    type ViewWith = SceneViewData<'a>;

    fn run_mut(&mut self, view: View, data: Self::ViewWith) {
        let dir_matrix = math::Matrix3::from_cols(
            self.view_matrix.x.truncate(),
            self.view_matrix.y.truncate(),
            self.view_matrix.z.truncate(),
        );

        unsafe {
            for v in view {
                if let &SceneNode::Light(light) = data.2.get_unchecked(v) {
                    match light.source {
                        LightSource::Directional => if self.data.dir.is_none() {
                            let dir = Transform::forward(&data.0, &data.1, v).unwrap();
                            let vdir = dir_matrix * dir;
                            let color: [f32; 4] = light.color.into();
                            self.data.dir = Some(RenderDataDirLight {
                                forward: dir,
                                dir: vdir,
                                dir_field: "u_DirLightEyeDir".into(),
                                color: math::Vector4::from(color).truncate(),
                                color_field: "u_DirLightColor".into(),
                            });
                        },

                        LightSource::Point { radius, smoothness } => {
                            let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                            let vp = (self.view_matrix * p.extend(1.0)).truncate();
                            let color: [f32; 4] = light.color.into();
                            let n = self.data.points.len();
                            self.data.points.push(RenderDataPointLight {
                                position: vp,
                                position_field: format!("u_PointLightEyePos[{0}]", n),
                                color: math::Vector4::from(color).truncate(),
                                color_field: format!("u_PointLightColor[{0}]", n),
                                attenuation: math::Vector3::new(
                                    1.0,
                                    -1.0 / (radius + smoothness * radius * radius),
                                    -smoothness / (radius + smoothness * radius * radius),
                                ),
                                attenuation_field: format!("u_PointLightAttenuation[{0}]", n),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct DrawOrder {
    tranlucent: bool,
    zorder: u32,
    shader: ShaderHandle,
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
