use crayon::ecs::{Arena, Entity, Fetch, System, View, World};
use crayon::application::Context;

use crayon::math;
use crayon::math::SquareMatrix;

use element::{Element, LitSrc};
use node::Node;
use transform::Transform;
use errors::*;

#[derive(Debug, Copy, Clone)]
pub struct EnvDirLit {
    pub handle: Entity,
    pub dir: math::Vector3<f32>,
    pub color: math::Vector3<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct EnvPointLit {
    pub position: math::Vector3<f32>,
    pub color: math::Vector3<f32>,
    pub attenuation: math::Vector3<f32>,
}

/// A trivial `RenderGraph` with brutal force iteration.
#[derive(Debug, Clone)]
pub struct RenderEnvironment {
    view_matrix: math::Matrix4<f32>,
    dirs: Vec<EnvDirLit>,
    points: Vec<EnvPointLit>,
    shadow_caster: Option<Entity>,
}

impl RenderEnvironment {
    pub fn new(_ctx: &Context) -> Result<Self> {
        Ok(RenderEnvironment {
            dirs: Vec::new(),
            points: Vec::new(),
            shadow_caster: None,
            view_matrix: math::Matrix4::identity(),
        })
    }

    /// Builds the `RenderGraph` from main camera.
    pub fn build(&mut self, world: &World, camera: Entity) -> Result<()> {
        let view_matrix = {
            let tree = world.arena::<Node>();
            let arena = world.arena::<Transform>();
            Transform::world_view_matrix(&tree, &arena, camera)?
        };

        self.view_matrix = view_matrix;
        UpdateRenderGraph::new(self).run_mut_at(world)
    }

    /// Gets the shadow caster.
    pub fn shadow_caster(&self) -> Option<Entity> {
        self.shadow_caster
    }

    ///
    pub fn dir_lits<T1>(&self, _: T1) -> Option<EnvDirLit>
    where
        T1: Into<math::Vector3<f32>>,
    {
        if self.dirs.is_empty() {
            None
        } else {
            Some(self.dirs[0])
        }
    }

    ///
    pub fn point_lits<T1>(&self, _: T1, _: f32) -> Vec<EnvPointLit>
    where
        T1: Into<math::Vector3<f32>>,
    {
        self.points.clone()
    }
}

struct UpdateRenderGraph<'a> {
    env: &'a mut RenderEnvironment,
}

impl<'a> UpdateRenderGraph<'a> {
    pub fn new(env: &'a mut RenderEnvironment) -> Self {
        UpdateRenderGraph { env: env }
    }
}

impl<'a, 'b> System<'a> for UpdateRenderGraph<'b> {
    type ViewWith = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, Element>);
    type Result = Result<()>;

    fn run_mut(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        let dir_matrix = math::Matrix3::from_cols(
            self.env.view_matrix.x.truncate(),
            self.env.view_matrix.y.truncate(),
            self.env.view_matrix.z.truncate(),
        );

        self.env.shadow_caster = None;
        self.env.dirs.clear();
        self.env.points.clear();

        unsafe {
            for v in view {
                if let Element::Light(lit) = *data.2.get_unchecked(v) {
                    if !lit.enable {
                        continue;
                    }

                    match lit.source {
                        LitSrc::Dir => {
                            if lit.shadow_caster {
                                self.env.shadow_caster = Some(v);
                            }

                            let dir = Transform::forward(&data.0, &data.1, v).unwrap();
                            let vdir = dir_matrix * dir;
                            let color: [f32; 4] = lit.color.into();

                            self.env.dirs.push(EnvDirLit {
                                handle: v,
                                dir: vdir,
                                color: math::Vector4::from(color).truncate(),
                            });
                        }

                        LitSrc::Point { radius, smoothness } => {
                            let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                            let vp = (self.env.view_matrix * p.extend(1.0)).truncate();
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
