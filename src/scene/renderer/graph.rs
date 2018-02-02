use std::sync::Arc;

use ecs::{Arena, Entity, Fetch, System, View, World};
use application::Context;

use math;
use math::{Matrix, SquareMatrix};
use graphics::{DrawCall, GraphicsSystem, GraphicsSystemShared, ShaderHandle, SurfaceHandle,
               UniformVariable};

use scene::{LitSrc, Material, MaterialHandle, Node, RenderUniform, SceneNode, Transform};
use scene::errors::*;

use utils::HandleObjectPool;

/// A trivial `RenderGraph` with brutal force iteration.
pub struct RenderGraph {
    video: Arc<GraphicsSystemShared>,
    pub dirs: Vec<EnvDirLit>,
    pub points: Vec<EnvPointLit>,
}

impl RenderGraph {
    pub fn new(ctx: &Context) -> Result<Self> {
        let video = ctx.shared::<GraphicsSystem>().clone();
        Ok(RenderGraph {
            video: video,
            dirs: Vec::new(),
            points: Vec::new(),
        })
    }

    /// Renders `RenderGraph`.
    pub fn render(
        &mut self,
        world: &World,
        mats: &HandleObjectPool<Material>,
        fallback: &Material,
        surface: SurfaceHandle,
        camera: Entity,
    ) -> Result<()> {
        let (view, projection) = {
            if let Some(SceneNode::Camera(v)) = world.get::<SceneNode>(camera) {
                let tree = world.arena::<Node>();
                let arena = world.arena::<Transform>();
                let view = Transform::world_view_matrix(&tree, &arena, camera)?;
                let projection = v.matrix();
                (view, projection)
            } else {
                bail!(ErrorKind::NonCameraFound);
            }
        };

        UpdateRenderGraph::new(self, view).run_mut_at(world)?;

        DrawRenderGraph {
            surface: surface,
            view_matrix: view,
            projection_matrix: projection,
            fallback: fallback,

            video: &self.video,
            materials: mats,
            env: self,
        }.run_mut_at(world)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EnvDirLit {
    pub dir: math::Vector3<f32>,
    pub color: math::Vector3<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct EnvPointLit {
    pub position: math::Vector3<f32>,
    pub color: math::Vector3<f32>,
    pub attenuation: math::Vector3<f32>,
}

struct UpdateRenderGraph<'a> {
    view_matrix: math::Matrix4<f32>,
    env: &'a mut RenderGraph,
}

impl<'a> UpdateRenderGraph<'a> {
    pub fn new(env: &'a mut RenderGraph, view: math::Matrix4<f32>) -> Self {
        UpdateRenderGraph {
            view_matrix: view,
            env: env,
        }
    }
}

impl<'a, 'b> System<'a> for UpdateRenderGraph<'b> {
    type ViewWith = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, SceneNode>);
    type Result = Result<()>;

    fn run_mut(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        let dir_matrix = math::Matrix3::from_cols(
            self.view_matrix.x.truncate(),
            self.view_matrix.y.truncate(),
            self.view_matrix.z.truncate(),
        );

        unsafe {
            for v in view {
                if let SceneNode::Light(lit) = *data.2.get_unchecked(v) {
                    match lit.source {
                        LitSrc::Dir => {
                            let dir = Transform::forward(&data.0, &data.1, v).unwrap();
                            let vdir = dir_matrix * dir;
                            let color: [f32; 4] = lit.color.into();

                            self.env.dirs.push(EnvDirLit {
                                dir: vdir,
                                color: math::Vector4::from(color).truncate(),
                            });
                        }

                        LitSrc::Point { radius, smoothness } => {
                            let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                            let vp = (self.view_matrix * p.extend(1.0)).truncate();
                            let color: [f32; 4] = lit.color.into();

                            self.env.points.push(EnvPointLit {
                                position: vp,
                                color: math::Vector4::from(color).truncate(),
                                attenuation: math::Vector3::new(
                                    1.0,
                                    -1.0 / (radius + smoothness * radius * radius),
                                    -smoothness / (radius + smoothness * radius * radius),
                                ),
                            })
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

struct DrawRenderGraph<'a> {
    surface: SurfaceHandle,
    projection_matrix: math::Matrix4<f32>,
    view_matrix: math::Matrix4<f32>,

    video: &'a GraphicsSystemShared,
    materials: &'a HandleObjectPool<Material>,
    fallback: &'a Material,
    env: &'a RenderGraph,
}

impl<'a> DrawRenderGraph<'a> {
    fn material(&self, handle: MaterialHandle) -> &Material {
        if let Some(mat) = self.materials.get(handle) {
            if self.video.shader_alive(mat.shader()) {
                return mat;
            }
        }

        self.fallback
    }

    fn bind_render_uniform<T>(dc: &mut DrawCall, m: &Material, uniform: RenderUniform, v: T)
    where
        T: Into<UniformVariable>,
    {
        let field = m.render_uniform_field(uniform);
        if m.has_uniform_variable(field) {
            dc.set_uniform_variable(field, v);
        }
    }
}

impl<'a, 'b> System<'a> for DrawRenderGraph<'b> {
    type ViewWith = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, SceneNode>);
    type Result = Result<()>;

    fn run_mut(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        use self::RenderUniform as RU;

        unsafe {
            for v in view {
                if let SceneNode::Mesh(mesh) = *data.2.get_unchecked(v) {
                    let mat = self.material(mesh.material);

                    let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                    let mut csp = self.view_matrix * math::Vector4::new(p.x, p.y, p.z, 1.0);
                    csp /= csp.w;

                    if csp.z <= 0.0 {
                        continue;
                    }

                    // Generate packed draw order.
                    let order = DrawOrder {
                        tranlucent: mat.render_state().color_blend.is_some(),
                        zorder: (csp.z * 1000.0) as u32,
                        shader: mat.shader(),
                    };

                    // Generate draw call and fill it with build-in uniforms.
                    let mut dc = DrawCall::new(mat.shader(), mesh.mesh);
                    for (k, v) in &mat.variables {
                        dc.set_uniform_variable(*k, *v);
                    }

                    let m = Transform::world_matrix(&data.0, &data.1, v).unwrap();
                    let mv = self.view_matrix * m;
                    let mvp = self.projection_matrix * mv;
                    let n = mv.invert().and_then(|v| Some(v.transpose())).unwrap_or(mv);

                    // Model matrix.
                    Self::bind_render_uniform(&mut dc, mat, RU::ModelMatrix, m);
                    // Model view matrix.
                    Self::bind_render_uniform(&mut dc, mat, RU::ModelViewMatrix, mv);
                    // Mode view projection matrix.
                    Self::bind_render_uniform(&mut dc, mat, RU::ModelViewProjectionMatrix, mvp);
                    // Normal matrix.
                    Self::bind_render_uniform(&mut dc, mat, RU::ViewNormalMatrix, n);

                    if !self.env.dirs.is_empty() {
                        let v = self.env.dirs[0];
                        // The direction of directional light in view space.
                        Self::bind_render_uniform(&mut dc, mat, RU::DirLightViewDir, v.dir);
                        // The color of directional light.
                        Self::bind_render_uniform(&mut dc, mat, RU::DirLightColor, v.color);
                    }

                    let len = ::std::cmp::min(self.env.points.len(), RU::POINT_LIT_UNIFORMS.len());

                    for i in 0..len {
                        let v = &self.env.points[i];
                        let uniforms = RU::POINT_LIT_UNIFORMS[i];
                        // The position of point light in view space.
                        Self::bind_render_uniform(&mut dc, mat, uniforms[0], v.position);
                        // The color of point light in view space.
                        Self::bind_render_uniform(&mut dc, mat, uniforms[1], v.color);
                        // The attenuation of point light in view space.
                        Self::bind_render_uniform(&mut dc, mat, uniforms[2], v.attenuation);
                    }

                    // Submit.
                    let sdc = dc.build(mesh.index)?;
                    self.video.submit(self.surface, order, sdc)?;
                }
            }
        }

        Ok(())
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
        (u64::from(prefix) << 32) | u64::from(suffix)
    }
}
