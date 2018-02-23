mod environment;
use self::environment::RenderEnvironment;

mod shadow;
use self::shadow::RenderShadow;

use crayon::application::Context;
use crayon::ecs::{Arena, Entity, Fetch, System, View, World};
use crayon::{graphics, math};
use crayon::math::{Matrix, SquareMatrix};

use assets::*;
use errors::*;
use element::*;

use node::Node;
use transform::Transform;
use scene::Scene;
use assets::pipeline::PipelineObject;
use assets::material::Material;

pub struct RendererSetup {
    pub max_dir_lits: usize,
    pub max_point_lits: usize,
    pub max_shadow_casters: usize,
}

pub struct Renderer {
    graph: RenderEnvironment,
    shadow: RenderShadow,
    ssm: math::Matrix4<f32>,
}

impl Renderer {
    pub fn new(ctx: &Context) -> Result<Renderer> {
        Ok(Renderer {
            graph: RenderEnvironment::new(ctx)?,
            shadow: RenderShadow::new(ctx)?,
            ssm: math::Matrix4::identity(),
        })
    }

    pub fn advance(&mut self, world: &World, camera: Entity) -> Result<()> {
        self.graph.build(world, camera)?;

        self.ssm = if let Some(caster) = self.graph.shadow_caster() {
            self.shadow.build_shadow_texture(world, caster)?
        } else {
            math::Matrix4::identity()
        };

        Ok(())
    }

    pub fn draw_shadow(&self, surface: graphics::SurfaceHandle) -> Result<()> {
        self.shadow.draw(surface)
    }

    pub fn draw(
        &self,
        scene: &Scene,
        surface: graphics::SurfaceHandle,
        camera: Entity,
    ) -> Result<()> {
        DrawTask {
            surface: surface,
            camera: camera,
            shadow_space_matrix: self.ssm,
            shadow_texture: self.shadow.texture(),
            scene: &scene,
            env: &self.graph,
        }.run_at(&scene.world)
    }
}

struct DrawTask<'a> {
    surface: graphics::SurfaceHandle,
    camera: Entity,

    shadow_space_matrix: math::Matrix4<f32>,
    shadow_texture: graphics::RenderTextureHandle,

    scene: &'a Scene,
    env: &'a RenderEnvironment,
}

impl<'a> DrawTask<'a> {
    fn material(&self, handle: MaterialHandle) -> (&PipelineObject, &Material) {
        if let Some(mat) = self.scene.materials.get(handle) {
            if let Some(pipeline) = self.scene.pipelines.get(*mat.pipeline) {
                if self.scene.video.shader_alive(pipeline.shader) {
                    return (pipeline, mat);
                }
            }
        }

        let mat = self.scene
            .materials
            .get(self.scene.fallback.unwrap())
            .unwrap();
        let pipeline = self.scene.pipelines.get(*mat.pipeline).unwrap();
        (pipeline, mat)
    }

    fn bind<T>(
        dc: &mut graphics::DrawCall,
        pipeline: &PipelineObject,
        uniform: PipelineUniformVariable,
        v: T,
    ) where
        T: Into<graphics::UniformVariable>,
    {
        let field = pipeline.uniform_field(uniform);
        if pipeline.sso.uniform_variable(field).is_some() {
            dc.set_uniform_variable(field, v);
        }
    }
}

impl<'a, 'b> System<'a> for DrawTask<'b> {
    type ViewWith = (Fetch<'a, Node>, Fetch<'a, Transform>, Fetch<'a, Element>);
    type Result = Result<()>;

    fn run(&self, view: View, data: Self::ViewWith) -> Self::Result {
        use assets::PipelineUniformVariable as RU;

        let (view_matrix, projection_matrix) = {
            if let Some(Element::Camera(v)) = self.scene.world.get::<Element>(self.camera) {
                let tree = self.scene.world.arena::<Node>();
                let arena = self.scene.world.arena::<Transform>();
                let view = Transform::world_view_matrix(&tree, &arena, self.camera)?;
                let projection = v.matrix();
                (view, projection)
            } else {
                bail!(ErrorKind::NonCameraFound);
            }
        };

        unsafe {
            for v in view {
                if let Element::Mesh(mesh) = *data.2.get_unchecked(v) {
                    let (pipeline, mat) = self.material(mesh.material);

                    let p = Transform::world_position(&data.0, &data.1, v).unwrap();
                    let mut csp = view_matrix * math::Vector4::new(p.x, p.y, p.z, 1.0);
                    csp /= csp.w;

                    if csp.z <= 0.0 {
                        continue;
                    }

                    // Generate packed draw order.
                    let order = DrawOrder {
                        tranlucent: pipeline.sso.render_state().color_blend.is_some(),
                        zorder: (csp.z * 1000.0) as u32,
                        shader: pipeline.shader,
                    };

                    // Generate draw call and fill it with build-in uniforms.
                    let mut dc = graphics::DrawCall::new(pipeline.shader, mesh.mesh);
                    for (k, v) in &mat.variables {
                        dc.set_uniform_variable(*k, *v);
                    }

                    let m = Transform::world_matrix(&data.0, &data.1, v).unwrap();
                    let mv = view_matrix * m;
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
                    // Shadow space matrix.
                    let ssm = self.shadow_space_matrix * m;
                    Self::bind(&mut dc, pipeline, RU::ShadowCasterSpaceMatrix, ssm);
                    // Shadow depth texture.
                    let st = self.shadow_texture;
                    Self::bind(&mut dc, pipeline, RU::ShadowTexture, st);

                    if let Some(dir) = self.env.dir_lits(p) {
                        // The direction of directional light in view space.
                        Self::bind(&mut dc, pipeline, RU::DirLightViewDir, dir.dir);
                        // The color of directional light.
                        Self::bind(&mut dc, pipeline, RU::DirLightColor, dir.color);
                    }

                    for (i, v) in self.env
                        .point_lits(p, 10.0)
                        .iter()
                        .enumerate()
                        .take(RU::POINT_LIT_UNIFORMS.len())
                    {
                        let uniforms = RU::POINT_LIT_UNIFORMS[i];
                        // The position of point light in view space.
                        Self::bind(&mut dc, pipeline, uniforms[0], v.position);
                        // The color of point light in view space.
                        Self::bind(&mut dc, pipeline, uniforms[1], v.color);
                        // The attenuation of point light in view space.
                        Self::bind(&mut dc, pipeline, uniforms[2], v.attenuation);
                    }

                    // Submit.
                    let sdc = dc.build(mesh.index)?;
                    self.scene.video.submit(self.surface, order, sdc)?;
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
        (u64::from(prefix) << 32) | u64::from(suffix)
    }
}
