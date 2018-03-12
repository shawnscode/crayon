use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::math;
use crayon::math::InnerSpace;

use components::prelude::*;
use errors::*;

#[derive(Debug, Copy, Clone)]
pub struct RenderLit {
    /// Handle of the lit in `world`.
    pub handle: Entity,
    /// Position in view space.
    pub position: math::Vector3<f32>,
    /// Direction in view space.
    pub dir: math::Vector3<f32>,
    /// The lit parameters.
    pub lit: Light,
    /// The shadow parameters.
    pub shadow: Option<RenderShadowCaster>,
}

#[derive(Debug, Copy, Clone)]
pub struct RenderShadowCaster {
    /// The matrix that transform from world space into shadow space.
    pub shadow_space_matrix: math::Matrix4<f32>,
    /// Objects that are beyond this distance from the camera will be rendered with
    /// no shadows at all.
    pub max_distance: f32,
}

/// A trivial `RenderGraph` with brutal force iteration.
#[derive(Debug, Clone)]
pub struct SimpleRenderGraph {
    lits: Vec<RenderLit>,
    visible_entities: Vec<Entity>,
}

impl SimpleRenderGraph {
    pub fn new(_: &Context) -> Result<Self> {
        Ok(SimpleRenderGraph {
            lits: Vec::new(),
            visible_entities: Vec::new(),
        })
    }

    /// Advances one frame with main camera.
    pub fn advance(&mut self, world: &World, camera: Entity) -> Result<()> {
        let (data, view_space_matrix) = if let Some(v) = world.get::<Camera>(camera) {
            let tree = world.arena::<Node>();
            let transforms = world.arena::<Transform>();
            let view = Transform::world_view_matrix(&tree, &transforms, camera)?;
            let projection = v.matrix();
            (*v, projection * view)
        } else {
            return Err(Error::NonCameraFound);
        };

        TaskGetRenderLits {
            graph: self,
            view_space_matrix: view_space_matrix,
        }.run_mut_at(world)?;

        TaskGetVisibleEntities {
            graph: self,
            camera: data,
            view_space_matrix: view_space_matrix,
        }.run_mut_at(world)?;
        Ok(())
    }

    /// Gets the iterator into lits.
    pub fn lits(&self) -> ::std::slice::Iter<RenderLit> {
        self.lits.iter()
    }

    /// Gets the visible entities from main camera.
    pub fn visible_entities(&self) -> ::std::slice::Iter<Entity> {
        self.visible_entities.iter()
    }
}

pub struct TaskGetVisibleEntities<'a> {
    graph: &'a mut SimpleRenderGraph,
    camera: Camera,
    view_space_matrix: math::Matrix4<f32>,
}

impl<'a, 'b> System<'a> for TaskGetVisibleEntities<'b> {
    type ViewWith = (
        Fetch<'a, Node>,
        Fetch<'a, Transform>,
        Fetch<'a, MeshRenderer>,
    );
    type Result = Result<()>;

    fn run_mut(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        self.graph.visible_entities.clear();

        unsafe {
            for v in view {
                let mesh = data.2.get_unchecked(v);
                if !mesh.visible {
                    continue;
                }

                let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                let vp = self.view_space_matrix * math::Vector4::new(p.x, p.y, p.z, 1.0);
                if vp.z <= self.camera.near_clip_plane() || vp.z > self.camera.far_clip_plane() {
                    continue;
                }

                self.graph.visible_entities.push(v);
            }
        }

        Ok(())
    }
}

pub struct TaskGetRenderLits<'a> {
    graph: &'a mut SimpleRenderGraph,
    view_space_matrix: math::Matrix4<f32>,
}

impl<'a, 'b> System<'a> for TaskGetRenderLits<'b> {
    type ViewWith = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, Light>);
    type Result = Result<()>;

    fn run_mut(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        unsafe {
            let dir_matrix = math::Matrix3::from_cols(
                self.view_space_matrix.x.truncate(),
                self.view_space_matrix.y.truncate(),
                self.view_space_matrix.z.truncate(),
            );

            self.graph.lits.clear();
            for v in view {
                let lit = data.2.get_unchecked(v);
                if !lit.enable {
                    continue;
                }

                let shadow = match lit.source {
                    LitSource::Dir => {
                        if lit.shadow_caster {
                            let view = Transform::world_view_matrix(&data.0, &data.1, v)?;
                            // FIXME: The projection frustum should be calculated based on the pose of camera.
                            let projection = Camera::ortho_matrix(-2.0, 2.0, -2.0, 2.0, 0.1, 10.0);
                            // FIXME: The max_distance should be configuratable.
                            Some(RenderShadowCaster {
                                shadow_space_matrix: projection * view,
                                max_distance: 100.0,
                            })
                        } else {
                            None
                        }
                    }
                    // FIXME: We should support shadow projector from point lit.
                    _ => None,
                };

                let position = Transform::world_position(&data.0, &data.1, v).unwrap();
                let dir = Transform::forward(&data.0, &data.1, v).unwrap();

                let rl = RenderLit {
                    handle: v,
                    position: (self.view_space_matrix * position.extend(1.0)).truncate(),
                    dir: (dir_matrix * dir).normalize(),
                    lit: *lit,
                    shadow: shadow,
                };

                self.graph.lits.push(rl);
            }
        }

        Ok(())
    }
}
