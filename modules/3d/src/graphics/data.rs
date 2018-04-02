use std::sync::Arc;
use std::collections::HashMap;

use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::math;
use crayon::math::{InnerSpace, SquareMatrix};
use crayon::graphics::prelude::*;

use components::prelude::*;
use errors::*;
use graphics::DrawSetup;

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
    /// The matrix that transform from world space into view space.
    pub shadow_view_matrix: math::Matrix4<f32>,
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

impl Default for RenderCamera {
    fn default() -> Self {
        let projection = math::Projection::Ortho {
            width: 128.0,
            height: 128.0,
            near: 0.1,
            far: 128.0,
        };

        let default_frustum = math::Frustum::new(projection);

        RenderCamera {
            id: Entity::nil(),
            component: Camera::default(),
            position: [0.0, 0.0, 0.0].into(),
            frustum: default_frustum,
            shadow_frustum: default_frustum,
            view_matrix: math::Matrix4::identity(),
        }
    }
}

pub struct RenderData {
    /// The active render camera.
    pub camera: RenderCamera,
    /// Entities which is visible from active render camera.
    pub visible_entities: Vec<Entity>,
    /// Enable lits in the scene.
    pub lits: Vec<RenderLit>,
    /// The world transforms.
    pub world_transforms: HashMap<Entity, Transform>,

    video: Arc<GraphicsSystemShared>,
}

impl RenderData {
    pub fn new(ctx: &Context) -> Self {
        RenderData {
            lits: Vec::new(),
            visible_entities: Vec::new(),
            camera: RenderCamera::default(),
            world_transforms: HashMap::new(),
            video: ctx.shared::<GraphicsSystem>().clone(),
        }
    }

    pub fn build(&mut self, world: &World, camera: Entity, setup: DrawSetup) -> Result<()> {
        if let Some(v) = world.get::<Camera>(camera) {
            let (_, nodes, transforms) = world.view_r2::<Node, Transform>();
            self.camera.id = camera;
            self.camera.component = *v;
            self.camera.position = Transform::world_position(&nodes, &transforms, camera)?;
            self.camera.view_matrix = Transform::world_view_matrix(&nodes, &transforms, camera)?;
        } else {
            return Err(Error::NonCameraFound);
        };

        {
            let (entities, nodes, transforms) = world.view_r2::<Node, Transform>();
            self.world_transforms = Transform::world_transforms(entities, &nodes, &transforms);
        }

        TaskGetVisibleEntities {
            world_transforms: &self.world_transforms,
            camera: &mut self.camera,
            visible_entities: &mut self.visible_entities,
            video: &self.video,
            setup: setup,
        }.run_with(world)?;

        TaskGetLitShadows {
            world_transforms: &self.world_transforms,
            camera: &self.camera,
            lits: &mut self.lits,
            setup: setup,
        }.run_with(world)?;

        Ok(())
    }
}

struct TaskGetVisibleEntities<'a> {
    world_transforms: &'a HashMap<Entity, Transform>,
    camera: &'a mut RenderCamera,
    visible_entities: &'a mut Vec<Entity>,
    video: &'a GraphicsSystemShared,
    setup: DrawSetup,
}

impl<'a, 'b> System<'a> for TaskGetVisibleEntities<'b> {
    type Data = (
        Fetch<'a, Node>,
        Fetch<'a, Transform>,
        Fetch<'a, MeshRenderer>,
    );
    type Err = Error;

    fn run(&mut self, entities: Entities, data: Self::Data) -> Result<()> {
        let cmp = self.camera.component;
        let view_matrix = self.camera.view_matrix;
        let frustum = math::Frustum::new(cmp.projection());
        let clip = (cmp.near_clip_plane(), cmp.far_clip_plane());
        let mut compact = (clip.1, clip.0);

        let video = &self.video;
        let world_transforms = &self.world_transforms;
        let v: Vec<_> = (entities, &data.0, &data.1, &data.2)
            .par_join(&entities, 128)
            .map(|(v, _, _, mesh)| {
                // Checks if mesh is visible.
                if !mesh.visible {
                    return None;
                }

                // Gets the underlying mesh params.
                let mso = if let Some(mso) = video.mesh(mesh.mesh) {
                    mso
                } else {
                    return None;
                };

                // Checks if mesh is visible for camera frustum.
                let model = world_transforms[&v].matrix();
                let aabb = mso.aabb.transform(&(view_matrix * model));
                if frustum.contains(&aabb) == math::PlaneRelation::Out {
                    return None;
                }

                Some((aabb, v))
            })
            .while_some()
            .collect();

        self.visible_entities.clear();
        self.visible_entities.extend(v.iter().map(|&(_, id)| id));

        compact = v.iter().map(|&(aabb, _)| aabb).fold(compact, |a, b| {
            (
                clip.0.max(a.0.min(b.min().z)),
                clip.1.min(a.1.max(b.max().z)),
            )
        });

        {
            let mut camera = self.camera.component;
            camera.set_clip_plane(compact.0 * 0.95, compact.1 * 1.05);
            self.camera.frustum = camera.frustum();
        }

        {
            compact.1 = compact.1.min(self.setup.max_shadow_distance);
            let mut camera = self.camera.component;
            camera.set_clip_plane(compact.0 * 0.99, compact.1 * 1.01);
            self.camera.shadow_frustum = camera.frustum();
        }

        Ok(())
    }
}

struct TaskGetLitShadows<'a> {
    world_transforms: &'a HashMap<Entity, Transform>,
    camera: &'a RenderCamera,
    lits: &'a mut Vec<RenderLit>,
    setup: DrawSetup,
}

impl<'a, 'b> System<'a> for TaskGetLitShadows<'b> {
    type Data = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, Light>);
    type Err = Error;

    fn run(&mut self, entities: Entities, data: Self::Data) -> Result<()> {
        let view_matrix = self.camera.view_matrix;
        let dir_matrix = math::Matrix3::from_cols(
            view_matrix.x.truncate(),
            view_matrix.y.truncate(),
            view_matrix.z.truncate(),
        );

        let icvm = Transform::inverse_world_view_matrix(&data.0, &data.1, self.camera.id)?;
        let frustum = self.camera.shadow_frustum;
        let frustum_points: math::FrustumPoints<_> = frustum.into();
        let max_shadow_distance = self.setup.max_shadow_distance;
        let world_transforms = &self.world_transforms;

        let v: Vec<_> = (entities, &data.0, &data.1, &data.2)
            .par_join(&entities, 128)
            .map(|(id, _, _, lit)| {
                if !lit.enable {
                    return None;
                }

                let shadow = match lit.source {
                    LitSource::Dir => {
                        if lit.shadow_caster {
                            // camera view space -> world space -> shadow view space
                            let v = Transform::world_view_matrix(&data.0, &data.1, id).unwrap();
                            let aabb = frustum_points.transform(&(v * icvm)).aabb();
                            let (mut min, max) = (aabb.min(), aabb.max());
                            if (max.z - min.z) > max_shadow_distance {
                                min.z = max.z - max_shadow_distance;
                            }
                            let dim = max - min;
                            let projection = math::Projection::ortho(dim.x, dim.y, min.z, max.z);
                            let frustum = math::Frustum::new(projection);
                            Some(RenderShadowCaster {
                                shadow_view_matrix: v,
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

                let transform = world_transforms[&id];
                Some(RenderLit {
                    handle: id,
                    position: (view_matrix * transform.position().extend(1.0)).truncate(),
                    dir: (dir_matrix * transform.forward()).normalize(),
                    lit: *lit,
                    shadow: shadow,
                })
            })
            .while_some()
            .collect();

        self.lits.clear();
        self.lits.extend(v.iter().cloned());

        Ok(())
    }
}
