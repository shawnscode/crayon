use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::math;
use crayon::math::{Matrix, SquareMatrix};
use crayon::rayon::prelude::*;
use crayon::utils::Handle;
use crayon::video::assets::prelude::*;
use crayon::video::prelude::*;

use assets::prelude::*;
use components::prelude::*;
use errors::*;

use graphics::data::RenderData;
use graphics::shadow::RenderShadow;
use graphics::DrawSetup;

use assets::material::Material;
use assets::pipeline::PipelineParams;
use resources::Resources;

pub struct Renderer {
    video: VideoSystemGuard,
    data: RenderData,

    shadow: RenderShadow,
    default_surface: SurfaceHandle,
    default_material: MaterialHandle,

    setup: DrawSetup,
}

impl Renderer {
    pub fn new(ctx: &Context, resources: &mut Resources, setup: DrawSetup) -> Result<Renderer> {
        let mut video = VideoSystemGuard::new(ctx.video.clone());
        let surface = video.create_surface(SurfaceParams::default())?;

        let undefined = factory::pipeline::undefined(resources)?;
        let mat = resources.create_material(MaterialSetup::new(undefined))?;

        Ok(Renderer {
            video: video,
            data: RenderData::new(ctx),
            shadow: RenderShadow::new(ctx)?,
            default_surface: surface,
            default_material: mat,
            setup: setup,
        })
    }

    pub fn advance(&mut self, world: &World, camera: Entity) -> Result<()> {
        self.data.build(world, camera, self.setup)?;
        self.shadow.build(world, &self.data, self.setup)?;
        Ok(())
    }

    pub fn draw_shadow(&self, surface: Option<SurfaceHandle>) -> Result<()> {
        for v in &self.data.lits {
            if v.shadow.is_some() {
                let surface = surface.unwrap_or(self.default_surface);
                return self.shadow.draw(v.handle, surface);
            }
        }

        Ok(())
    }

    pub fn draw(&self, world: &World, resources: &Resources) -> Result<()> {
        TaskDrawCall {
            resources: resources,
            renderer: self,
        }.run_with(world)
    }
}

struct TaskDrawCall<'a> {
    resources: &'a Resources,
    renderer: &'a Renderer,
}

impl<'a> TaskDrawCall<'a> {
    fn material(
        video: &VideoSystemShared,
        resources: &'a Resources,
        handle: MaterialHandle,
        fallback: MaterialHandle,
    ) -> (&'a PipelineParams, &'a Material) {
        if let Some(mat) = resources.materials.get(handle) {
            if let Some(pipeline) = resources.pipelines.get(mat.pipeline) {
                if video.is_shader_alive(pipeline.shader) {
                    return (pipeline, mat);
                }
            }
        }

        let mat = resources.materials.get(fallback).unwrap();
        let pipeline = resources.pipelines.get(mat.pipeline).unwrap();
        (pipeline, mat)
    }

    fn bind<T>(dc: &mut DrawCall, pipeline: &PipelineParams, uniform: PipelineUniformVariable, v: T)
    where
        T: Into<UniformVariable>,
    {
        let field = pipeline.uniform_field(uniform);
        if pipeline
            .shader_params
            .uniforms
            .variable_type(field)
            .is_some()
        {
            dc.set_uniform_variable(field, v);
        }
    }
}

impl<'a, 'b> System<'a> for TaskDrawCall<'b> {
    type Data = (
        Fetch<'a, Node>,
        Fetch<'a, Transform>,
        Fetch<'a, MeshRenderer>,
    );

    type Err = Error;

    fn run(&mut self, _: Entities, cmps: Self::Data) -> Result<()> {
        use self::PipelineUniformVariable as RU;

        let camera = self.renderer.data.camera;
        let surface = camera
            .component
            .surface()
            .unwrap_or(self.renderer.default_surface);
        let projection_matrix = camera.frustum.to_matrix();

        let data = &self.renderer.data;
        let video = &self.renderer.video;
        let default_material = self.renderer.default_material;
        let resources = &self.resources;

        self.renderer
            .data
            .visible_entities
            .par_iter()
            .for_each(|&id| {
                let mesh = if let Some(v) = cmps.2.get(id) {
                    v
                } else {
                    return;
                };

                let transform = data.world_transforms[&id];

                // Gets the underlying mesh params.
                let mso = if let Some(mso) = video.mesh(mesh.mesh) {
                    if mso.sub_mesh_offsets.len() <= 0 {
                        return;
                    }
                    mso
                } else {
                    return;
                };

                let fallback = mesh.materials
                    .get(0)
                    .cloned()
                    .unwrap_or(Handle::nil().into());

                // Iterates and draws the sub-meshes with corresponding material.
                let mut from = mso.sub_mesh_offsets[0];
                let mut current_mat = fallback;

                for i in 0..mso.sub_mesh_offsets.len() {
                    let len = if i == mso.sub_mesh_offsets.len() - 1 {
                        mso.num_idxes - from
                    } else {
                        let mat = mesh.materials.get(i + 1).cloned().unwrap_or(fallback);
                        if current_mat == mat {
                            continue;
                        }

                        mso.sub_mesh_offsets[i + 1] - from
                    };

                    let (pipeline, mat) =
                        Self::material(video, resources, current_mat, default_material);
                    let mut csp = camera.view_matrix * transform.position().extend(1.0);
                    csp /= csp.w;

                    // Generate packed draw order.
                    let order = DrawCommandOrder {
                        tranlucent: pipeline.shader_params.render_state.color_blend.is_some(),
                        zorder: (csp.z * 1000.0) as u32,
                        shader: pipeline.shader,
                    };

                    // Generate draw call and fill it with build-in uniforms.
                    let mut dc = DrawCall::new(pipeline.shader, mesh.mesh);
                    for (k, v) in &mat.variables {
                        dc.set_uniform_variable(*k, *v);
                    }

                    let m = transform.matrix();
                    let mv = camera.view_matrix * m;
                    let mvp = projection_matrix * mv;
                    let n = mv.invert().and_then(|v| Some(v.transpose())).unwrap_or(mv);

                    // Model matrix.
                    Self::bind(&mut dc, pipeline, RU::ModelMatrix, m);
                    // Model view matrix.
                    Self::bind(&mut dc, pipeline, RU::ModelViewMatrix, mv);
                    // Mode view projection matrix.
                    Self::bind(&mut dc, pipeline, RU::ModelViewProjectionMatrix, mvp);
                    // Normal matrix.
                    Self::bind(&mut dc, pipeline, RU::ViewNormalMatrix, n);

                    // FIXME: The lits that affected object should be determined by the distance.
                    let (mut dir_index, mut point_index) = (0, 0);
                    for l in &self.renderer.data.lits {
                        let color: [f32; 4] = l.lit.color.rgba();
                        let color = [color[0], color[1], color[2]];

                        match l.lit.source {
                            LitSource::Dir => if dir_index < RU::DIR_LIT_UNIFORMS.len() {
                                let uniforms = RU::DIR_LIT_UNIFORMS[dir_index];
                                // The direction of directional light in view space.
                                Self::bind(&mut dc, pipeline, uniforms[0], l.dir);
                                // The color of directional light.
                                Self::bind(&mut dc, pipeline, uniforms[1], color);

                                if let Some(shadow) = l.shadow {
                                    if let Some(rt) =
                                        self.renderer.shadow.depth_render_texture(l.handle)
                                    {
                                        // Shadow depth texture.
                                        Self::bind(&mut dc, pipeline, uniforms[2], rt);
                                        // Shadow space matrix.
                                        let ssm = shadow.shadow_space_matrix * m;
                                        Self::bind(&mut dc, pipeline, uniforms[3], ssm);
                                    }
                                }

                                dir_index += 1;
                            },

                            LitSource::Point { radius, smoothness } => {
                                if point_index < RU::POINT_LIT_UNIFORMS.len() {
                                    let uniforms = RU::POINT_LIT_UNIFORMS[point_index];
                                    // The position of point light in view space.
                                    Self::bind(&mut dc, pipeline, uniforms[0], l.position);
                                    // The color of point light in view space.
                                    Self::bind(&mut dc, pipeline, uniforms[1], color);

                                    let attenuation = math::Vector3::new(
                                        1.0,
                                        -1.0 / (radius + smoothness * radius * radius),
                                        -smoothness / (radius + smoothness * radius * radius),
                                    );

                                    // The attenuation of point light in view space.
                                    Self::bind(&mut dc, pipeline, uniforms[2], attenuation);
                                }

                                point_index += 1;
                            }
                        }
                    }

                    // Submit.
                    let sdc = dc.build_from(from, len).unwrap();
                    video.submit(surface, order, sdc).unwrap();

                    //
                    from = from + len;
                    current_mat = mesh.materials.get(i + 1).cloned().unwrap_or(fallback);
                }
            });

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct DrawCommandOrder {
    tranlucent: bool,
    zorder: u32,
    shader: ShaderHandle,
}

impl Into<u64> for DrawCommandOrder {
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
