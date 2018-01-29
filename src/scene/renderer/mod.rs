pub mod uniforms;
pub use self::uniforms::SceneUniformVariables;

use self::SceneUniformVariables as SUV;

use std::collections::HashMap;

use ecs::{Arena, Fetch, System, View};
use math;
use math::{Matrix, SquareMatrix};
use graphics::{DrawCall, GraphicsSystemShared, MeshHandle, MeshIndex, ShaderHandle, SurfaceHandle,
               UniformVariable};
use utils::{HandleObjectPool, HashValue};

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
    pub dir: math::Vector3<f32>,
    pub color: math::Vector3<f32>,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderDataPointLight {
    /// Position in eye space.
    pub position: math::Vector3<f32>,
    pub color: math::Vector3<f32>,
    pub attenuation: math::Vector3<f32>,
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
    pub shader_binds: &'a HashMap<ShaderHandle, HashMap<SceneUniformVariables, HashValue<str>>>,
    pub surface: SurfaceHandle,
    pub view_matrix: math::Matrix4<f32>,
    pub projection_matrix: math::Matrix4<f32>,
    pub data: RenderData,
}

fn bind<T>(
    dc: &mut DrawCall,
    m: &Material,
    binds: &HashMap<SceneUniformVariables, HashValue<str>>,
    suv: SceneUniformVariables,
    v: T,
) where
    T: Into<UniformVariable>,
{
    let field = binds.get(&suv).and_then(|v| Some(*v)).unwrap_or(suv.into());
    if m.has_uniform_variable(field) {
        dc.set_uniform_variable(field, v);
    }
}

impl<'a, 'b> System<'a> for RenderTask<'b> {
    type ViewWith = SceneViewData<'a>;

    fn run(&self, view: View, data: Self::ViewWith) {
        unsafe {
            for v in view {
                if let &SceneNode::Mesh(mesh) = data.2.get_unchecked(v) {
                    let mut mat = self.materials.get(mesh.material).unwrap_or(self.fallback);
                    if !self.video.shader_alive(mat.shader()) {
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
                    let binds = self.shader_binds.get(&mat.shader()).unwrap();

                    for (k, v) in &mat.variables {
                        dc.set_uniform_variable(*k, *v);
                    }

                    let m = Transform::world_matrix(&data.0, &data.1, v).unwrap();
                    let mv = self.view_matrix * m;
                    let mvp = self.projection_matrix * mv;
                    let n = mv.invert().and_then(|v| Some(v.transpose())).unwrap_or(mv);

                    // Model matrix.
                    bind(&mut dc, &mat, &binds, SUV::ModelMatrix, m);
                    // Model view matrix.
                    bind(&mut dc, &mat, &binds, SUV::ModelViewMatrix, mv);
                    // Mode view projection matrix.
                    bind(&mut dc, &mat, &binds, SUV::ModelViewProjectionMatrix, mvp);
                    // Normal matrix.
                    bind(&mut dc, &mat, &binds, SUV::ViewNormalMatrix, n);

                    if let Some(ref dir) = self.data.dir {
                        // The direction of directional light in view space.
                        bind(&mut dc, &mat, &binds, SUV::DirLightViewDir, dir.dir);
                        // The color of directional light.
                        bind(&mut dc, &mat, &binds, SUV::DirLightColor, dir.color);
                    }

                    let len = ::std::cmp::min(self.data.points.len(), SUV::POINT_LIT_FIELDS.len());

                    for i in 0..len {
                        let v = &self.data.points[i];
                        let fields = SUV::POINT_LIT_FIELDS[i];
                        // The position of point light in view space.
                        bind(&mut dc, &mat, &binds, fields[0], v.position);
                        // The color of point light in view space.
                        bind(&mut dc, &mat, &binds, fields[1], v.color);
                        // The attenuation of point light in view space.
                        bind(&mut dc, &mat, &binds, fields[2], v.attenuation);
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
                                dir: vdir,
                                color: math::Vector4::from(color).truncate(),
                            });
                        },

                        LightSource::Point { radius, smoothness } => {
                            let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                            let vp = (self.view_matrix * p.extend(1.0)).truncate();
                            let color: [f32; 4] = light.color.into();
                            self.data.points.push(RenderDataPointLight {
                                position: vp,
                                color: math::Vector4::from(color).truncate(),
                                attenuation: math::Vector3::new(
                                    1.0,
                                    -1.0 / (radius + smoothness * radius * radius),
                                    -smoothness / (radius + smoothness * radius * radius),
                                ),
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
