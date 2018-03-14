use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::math;
use crayon::math::{Matrix, SquareMatrix};
use crayon::graphics::prelude::*;
use crayon::graphics::assets::prelude::*;
use crayon::utils::Handle;

use components::prelude::*;
use assets::prelude::*;
use errors::*;

use graphics::shadow::RenderShadow;
use graphics::graph::SimpleRenderGraph;

use scene::Scene;
use assets::material::Material;
use assets::pipeline::PipelineParams;

pub struct RendererSetup {
    pub max_dir_lits: usize,
    pub max_point_lits: usize,
    pub max_shadow_casters: usize,
}

pub struct Renderer {
    video: GraphicsSystemGuard,
    graph: SimpleRenderGraph,
    shadow: RenderShadow,
    default_surface: SurfaceHandle,
}

impl Renderer {
    pub fn new(ctx: &Context) -> Result<Renderer> {
        let mut video = GraphicsSystemGuard::new(ctx.shared::<GraphicsSystem>().clone());
        let setup = SurfaceSetup::default();
        let surface = video.create_surface(setup)?;

        Ok(Renderer {
            video: video,
            graph: SimpleRenderGraph::new(ctx)?,
            shadow: RenderShadow::new(ctx)?,
            default_surface: surface,
        })
    }

    pub fn advance(&mut self, world: &World, camera: Entity) -> Result<()> {
        self.graph.advance(world, camera)?;
        self.shadow.advance(world, &self.graph)?;
        Ok(())
    }

    pub fn draw_shadow(&self, surface: Option<SurfaceHandle>) -> Result<()> {
        for v in self.graph.lits() {
            if v.shadow.is_some() {
                let surface = surface.unwrap_or(self.default_surface);
                return self.shadow.draw(v.handle, surface);
            }
        }

        Ok(())
    }

    pub fn draw(&self, scene: &Scene) -> Result<()> {
        TaskDraw {
            scene: scene,
            renderer: self,
            default_surface: self.default_surface,
        }.run_at(&scene.world)
    }
}

struct TaskDraw<'a> {
    scene: &'a Scene,
    renderer: &'a Renderer,
    default_surface: SurfaceHandle,
}

impl<'a> TaskDraw<'a> {
    fn material(&self, handle: MaterialHandle) -> (&PipelineParams, &Material) {
        if let Some(mat) = self.scene.materials.get(handle) {
            if let Some(pipeline) = self.scene.pipelines.get(mat.pipeline) {
                if self.scene.video.is_shader_alive(pipeline.shader) {
                    return (pipeline, mat);
                }
            }
        }

        let mat = self.scene
            .materials
            .get(self.scene.fallback.unwrap())
            .unwrap();

        let pipeline = self.scene.pipelines.get(mat.pipeline).unwrap();
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

impl<'a, 'b> System<'a> for TaskDraw<'b> {
    type ViewWith = (
        Fetch<'a, Node>,
        Fetch<'a, Transform>,
        Fetch<'a, MeshRenderer>,
    );
    type Result = Result<()>;

    fn run(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        use self::PipelineUniformVariable as RU;

        unsafe {
            let camera = self.renderer.graph.camera();
            let surface = camera.component.surface().unwrap_or(self.default_surface);
            let projection_matrix = camera.frustum.to_matrix();

            for v in view {
                let mesh = data.2.get_unchecked(v);

                // Gets the underlying mesh params.
                let mso = if let Some(mso) = self.scene.video.mesh(mesh.mesh) {
                    mso
                } else {
                    continue;
                };

                // Gets the model matrix.
                let model = Transform::world_matrix(&data.0, &data.1, v).unwrap();

                // Checks if mesh is visible.
                if !mesh.visible {
                    continue;
                }

                // let aabb = mso.aabb.transform(&model);
                // if !mesh.visible || camera.frustum.contains(&aabb) == math::PlaneRelation::Out {
                //     continue;
                // }

                // Iterates and draws the sub-meshes with corresponding material.
                for i in 0..mso.sub_mesh_offsets.len() {
                    let (pipeline, mat) =
                        self.material(*mesh.materials.get(i).unwrap_or(&Handle::nil().into()));

                    let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                    let mut csp = camera.view_matrix * math::Vector4::new(p.x, p.y, p.z, 1.0);
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

                    let mv = camera.view_matrix * model;
                    let mvp = projection_matrix * mv;
                    let n = mv.invert().and_then(|v| Some(v.transpose())).unwrap_or(mv);

                    // Model matrix.
                    Self::bind(&mut dc, pipeline, RU::ModelMatrix, model);
                    // Model view matrix.
                    Self::bind(&mut dc, pipeline, RU::ModelViewMatrix, mv);
                    // Mode view projection matrix.
                    Self::bind(&mut dc, pipeline, RU::ModelViewProjectionMatrix, mvp);
                    // Normal matrix.
                    Self::bind(&mut dc, pipeline, RU::ViewNormalMatrix, n);

                    // FIXME: The lits that affected object should be determined by the distance.
                    let (mut dir_index, mut point_index) = (0, 0);
                    for l in self.renderer.graph.lits() {
                        let color: [f32; 4] = l.lit.color.into();
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
                                        let ssm = shadow.shadow_space_matrix * model;
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
                    let sdc = dc.build_sub_mesh(i)?;
                    self.renderer.video.submit(surface, order, sdc)?;
                }
            }
        }

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
