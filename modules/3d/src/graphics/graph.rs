use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::math;
use crayon::math::{InnerSpace, SquareMatrix};
use crayon::graphics::prelude::*;

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
    /// The frustum that builds from `shadow_space_matrix`.
    pub shadow_frustum: math::Frustum<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct RenderCamera {
    /// The id of camera.
    pub id: Entity,
    /// The original camera component.
    pub component: Camera,
    /// The position of camera in world space.
    pub position: math::Vector3<f32>,
    /// The frustum in view space.
    pub frustum: math::Frustum<f32>,
    /// The frustum in view space that clipped with `shadow_max_distance`.
    pub shadow_frustum: math::Frustum<f32>,
    /// The view matrix.
    pub view_matrix: math::Matrix4<f32>,
}

/// A trivial `RenderGraph` with brutal force iteration.
pub struct SimpleRenderGraph {
    lits: Vec<RenderLit>,
    visible_entities: Vec<Entity>,
    video: GraphicsSystemGuard,

    /// The active render camera.
    camera: RenderCamera,
    /// Objects that are beyond this distance from the camera will be rendered with
    /// no shadows at all.
    shadow_max_distance: f32,
}

impl SimpleRenderGraph {
    pub fn new(ctx: &Context) -> Result<Self> {
        let video = GraphicsSystemGuard::new(ctx.shared::<GraphicsSystem>().clone());

        let projection = math::Projection::Ortho {
            width: 128.0,
            height: 128.0,
            near: 0.1,
            far: 128.0,
        };
        let default_frustum = math::Frustum::new(projection);

        let render_camera = RenderCamera {
            id: Entity::nil(),
            component: Camera::default(),
            position: [0.0, 0.0, 0.0].into(),
            frustum: default_frustum,
            shadow_frustum: default_frustum,
            view_matrix: math::Matrix4::identity(),
        };

        Ok(SimpleRenderGraph {
            lits: Vec::new(),
            visible_entities: Vec::new(),
            video: video,

            camera: render_camera,
            shadow_max_distance: 100.0,
        })
    }

    /// Advances one frame with main camera.
    pub fn advance(&mut self, world: &World, camera: Entity) -> Result<()> {
        if let Some(v) = world.get::<Camera>(camera) {
            let tree = world.arena::<Node>();
            let transforms = world.arena::<Transform>();

            self.camera.id = camera;
            self.camera.component = *v;
            self.camera.position = Transform::world_position(&tree, &transforms, camera)?;
            self.camera.view_matrix = Transform::world_view_matrix(&tree, &transforms, camera)?;
        } else {
            return Err(Error::NonCameraFound);
        };

        TaskGetVisibleEntities { graph: self }.run_at(world)?;
        TaskGetRenderLits { graph: self }.run_at(world)?;

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

    /// Gets the active camera.
    pub fn camera(&self) -> RenderCamera {
        self.camera
    }
}

pub struct TaskGetVisibleEntities<'a> {
    graph: &'a mut SimpleRenderGraph,
}

impl<'a, 'b> System<'a> for TaskGetVisibleEntities<'b> {
    type ViewWith = (
        Fetch<'a, Node>,
        Fetch<'a, Transform>,
        Fetch<'a, MeshRenderer>,
    );
    type Result = Result<()>;

    fn run(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        self.graph.visible_entities.clear();

        let cmp = self.graph.camera.component;
        let view_matrix = self.graph.camera.view_matrix;
        let frustum = math::Frustum::new(cmp.projection());

        let clip = (cmp.near_clip_plane(), cmp.far_clip_plane());
        let mut tight_clip = (clip.1, clip.0);

        unsafe {
            for v in view {
                let mesh = data.2.get_unchecked(v);

                // Checks if mesh is visible.
                if !mesh.visible {
                    continue;
                }

                // Gets the underlying mesh params.
                let mso = if let Some(mso) = self.graph.video.mesh(mesh.mesh) {
                    mso
                } else {
                    continue;
                };

                // Checks if mesh is visible for camera frustum.
                let model = Transform::world_matrix(&data.0, &data.1, v).unwrap();
                let aabb = mso.aabb.transform(&(view_matrix * model));
                if frustum.contains(&aabb) == math::PlaneRelation::Out {
                    continue;
                }

                tight_clip.0 = clip.0.max(tight_clip.0.min(aabb.min().z));
                tight_clip.1 = clip.1.min(tight_clip.1.max(aabb.max().z));
                self.graph.visible_entities.push(v);
            }
        }

        {
            let mut camera = self.graph.camera.component;
            camera.set_clip_plane(tight_clip.0 * 0.95, tight_clip.1 * 1.05);
            self.graph.camera.frustum = camera.frustum();
        }

        {
            tight_clip.1 = tight_clip.1.min(self.graph.shadow_max_distance);

            let mut camera = self.graph.camera.component;
            camera.set_clip_plane(tight_clip.0 * 0.99, tight_clip.1 * 1.01);
            self.graph.camera.shadow_frustum = camera.frustum();
        }

        Ok(())
    }
}

pub struct TaskGetRenderLits<'a> {
    graph: &'a mut SimpleRenderGraph,
}

impl<'a, 'b> System<'a> for TaskGetRenderLits<'b> {
    type ViewWith = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, Light>);
    type Result = Result<()>;

    fn run(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        unsafe {
            let view_matrix = self.graph.camera.view_matrix;
            let dir_matrix = math::Matrix3::from_cols(
                view_matrix.x.truncate(),
                view_matrix.y.truncate(),
                view_matrix.z.truncate(),
            );

            let id = self.graph.camera.id;
            let icvm = Transform::inverse_world_view_matrix(&data.0, &data.1, id)?;
            let frustum = self.graph.camera.shadow_frustum;
            let frustum_points: math::FrustumPoints<_> = frustum.into();

            self.graph.lits.clear();
            for v in view {
                let lit = data.2.get_unchecked(v);
                if !lit.enable {
                    continue;
                }

                let shadow = match lit.source {
                    LitSource::Dir => {
                        if lit.shadow_caster {
                            // camera view space -> world space -> shadow view space
                            let v = Transform::world_view_matrix(&data.0, &data.1, v)?;
                            let aabb = frustum_points.transform(&(v * icvm)).aabb();
                            let (mut min, max) = (aabb.min(), aabb.max());
                            if (max.z - min.z) > self.graph.shadow_max_distance {
                                min.z = max.z - self.graph.shadow_max_distance;
                            }

                            let dim = max - min;
                            let projection = math::Projection::ortho(dim.x, dim.y, min.z, max.z);
                            let frustum = math::Frustum::new(projection);

                            Some(RenderShadowCaster {
                                shadow_space_matrix: projection.to_matrix() * v,
                                shadow_frustum: frustum,
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
                    position: (view_matrix * position.extend(1.0)).truncate(),
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
