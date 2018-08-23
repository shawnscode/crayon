mod material;
pub use self::material::Material;

use crayon::application::Context;
use crayon::errors::*;
use crayon::math;
use crayon::video::assets::prelude::*;
use crayon::video::prelude::*;

use std::sync::Arc;

use super::{Camera, Lit, LitSource, MeshRenderer};
use {Component, Entity};

pub const MAX_DIR_LITS: usize = 1;
pub const MAX_POINT_LITS: usize = 4;

pub struct SimpleRenderPipeline {
    materials: Component<Material>,

    surface: SurfaceHandle,
    shader: ShaderHandle,
    video: Arc<VideoSystemShared>,
    drawcalls: OrderDrawBatch<DrawOrder>,

    dir_lits: Vec<(String, String)>,
    point_lits: Vec<(String, String, String)>,
}

impl SimpleRenderPipeline {
    pub fn new(ctx: &Context) -> Result<Self> {
        // Create shader state.
        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 3)
            .with(Attribute::Normal, 3)
            .finish();

        let mut uniforms = UniformVariableLayout::build()
            .with("u_ModelViewMatrix", UniformVariableType::Matrix4f)
            .with("u_MVPMatrix", UniformVariableType::Matrix4f)
            .with("u_ViewNormalMatrix", UniformVariableType::Matrix4f)
            .with("u_Ambient", UniformVariableType::Vector3f)
            .with("u_Diffuse", UniformVariableType::Vector3f)
            .with("u_Specular", UniformVariableType::Vector3f)
            .with("u_Shininess", UniformVariableType::F32);
        // .with("u_Texture", UniformVariableType::Texture);

        let mut dir_lits = Vec::new();
        let mut point_lits = Vec::new();

        for i in 0..MAX_DIR_LITS {
            let name = (
                format!("u_DirLitViewDir[{0}]", i),
                format!("u_DirLitColor[{0}]", i),
            );

            uniforms = uniforms
                .with(name.0.as_ref(), UniformVariableType::Matrix4f)
                .with(name.1.as_ref(), UniformVariableType::Matrix4f);

            dir_lits.push(name);
        }

        for i in 0..MAX_POINT_LITS {
            let name = (
                format!("u_PointLitViewPos[{0}]", i),
                format!("u_PointLitColor[{0}]", i),
                format!("u_PointLitAttenuation[{0}]", i),
            );

            uniforms = uniforms
                .with(name.0.as_ref(), UniformVariableType::Matrix4f)
                .with(name.1.as_ref(), UniformVariableType::Matrix4f)
                .with(name.2.as_ref(), UniformVariableType::Matrix4f);

            point_lits.push(name);
        }

        let mut params = ShaderParams::default();
        params.state.depth_write = true;
        params.state.depth_test = Comparison::Less;
        params.attributes = attributes;
        params.uniforms = uniforms.finish();

        let vs = format!(
            "
            #version 100
            precision lowp float;

            #define MAX_DIR_LITS {0}
            #define MAX_POINT_LITS {1}
            {2}
            ",
            MAX_DIR_LITS,
            MAX_POINT_LITS,
            include_str!("../../../assets/simple.vs")
        );

        let fs = format!(
            "
            #version 100
            precision lowp float;

            #define MAX_DIR_LITS {0}
            #define MAX_POINT_LITS {1}
            {2}
            ",
            MAX_DIR_LITS,
            MAX_POINT_LITS,
            include_str!("../../../assets/simple.fs")
        );

        let shader = ctx.video.create_shader(params, vs, fs)?;

        let params = SurfaceParams::default();
        let surface = ctx.video.create_surface(params)?;

        Ok(SimpleRenderPipeline {
            materials: Component::new(),
            video: ctx.video.clone(),
            surface: surface,
            shader: shader,
            drawcalls: OrderDrawBatch::new(),
            dir_lits: dir_lits,
            point_lits: point_lits,
        })
    }

    #[inline]
    pub fn add(&mut self, ent: Entity, material: Material) -> Option<Material> {
        self.materials.add(ent, material)
    }

    #[inline]
    pub fn has(&self, ent: Entity) -> bool {
        self.materials.has(ent)
    }

    #[inline]
    pub fn material(&self, ent: Entity) -> Option<&Material> {
        self.materials.get(ent)
    }

    #[inline]
    pub fn material_mut(&mut self, ent: Entity) -> Option<&mut Material> {
        self.materials.get_mut(ent)
    }

    #[inline]
    pub fn remove(&mut self, ent: Entity) {
        self.materials.remove(ent)
    }
}

impl super::RenderPipeline for SimpleRenderPipeline {
    fn submit(&mut self, camera: &Camera, lits: &[Lit], meshes: &[MeshRenderer]) {
        use crayon::math::{Matrix, MetricSpace, SquareMatrix};

        let view_matrix = camera.transform.view_matrix();
        let projection_matrix = camera.frustum().to_matrix();
        let mut lits = Vec::from(lits);

        for mesh in meshes {
            let model_matrix = mesh.transform.matrix();
            let mv = view_matrix * model_matrix;
            let mvp = projection_matrix * mv;
            let vn = mv.invert().and_then(|v| Some(v.transpose())).unwrap_or(mv);

            let mut dc = DrawCall::new(self.shader, mesh.mesh);
            dc.set_uniform_variable("u_ModelViewMatrix", mv);
            dc.set_uniform_variable("u_MVPMatrix", mvp);
            dc.set_uniform_variable("u_ViewNormalMatrix", vn);

            let mat = self.material(mesh.ent).cloned().unwrap_or_default();
            dc.set_uniform_variable("u_Ambient", mat.ambient.rgb());
            dc.set_uniform_variable("u_Diffuse", mat.diffuse.rgb());
            dc.set_uniform_variable("u_Specular", mat.specular.rgb());
            dc.set_uniform_variable("u_Shininess", mat.shininess);

            lits.sort_by_key(|v| mesh.transform.position.distance2(v.transform.position) as u32);

            let (mut dir_index, mut point_index) = (0, 0);
            for lit in &lits {
                match lit.source {
                    LitSource::Dir => {
                        if dir_index < self.dir_lits.len() {
                            let names = &self.dir_lits[dir_index];
                            let dir = view_matrix * lit.transform.forward().extend(1.0);
                            dc.set_uniform_variable(&names.0, dir.truncate());
                            dc.set_uniform_variable(&names.1, lit.color.rgb());
                            dir_index += 1;
                        }
                    }
                    LitSource::Point { radius, smoothness } => {
                        if point_index < self.point_lits.len() {
                            let names = &self.point_lits[point_index];
                            let pos = view_matrix * lit.transform.position.extend(1.0);
                            let attenuation = math::Vector3::new(
                                1.0,
                                -1.0 / (radius + smoothness * radius * radius),
                                -smoothness / (radius + smoothness * radius * radius),
                            );
                            dc.set_uniform_variable(&names.0, pos.truncate());
                            dc.set_uniform_variable(&names.1, lit.color.rgb());
                            dc.set_uniform_variable(&names.2, attenuation);

                            point_index += 1;
                        }
                    }
                }
            }

            let order = DrawOrder::new(
                self.shader,
                false,
                mesh.transform.position.distance2(camera.transform.position) as u32,
            );

            self.drawcalls.draw(order, dc);
        }

        let surface = camera.surface().unwrap_or(self.surface);
        self.drawcalls.submit(&self.video, surface).unwrap();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DrawOrder(u64);

impl DrawOrder {
    fn new(shader: ShaderHandle, translucent: bool, zorder: u32) -> Self {
        let prefix = if translucent { (!zorder) } else { zorder };
        let suffix = shader.index();
        DrawOrder((u64::from(prefix) << 32) | u64::from(suffix))
    }
}
