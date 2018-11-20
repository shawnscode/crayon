mod material;
pub use self::material::SimpleMaterial;

use crayon::prelude::*;
use failure::Error;

use utils::prelude::Component;
use Entity;

use super::{Camera, Lit, LitSource, MeshRenderer};

pub const MAX_DIR_LITS: usize = 1;
pub const MAX_POINT_LITS: usize = 4;

/// A simple renderer that draws some color into mesh objects.
pub struct SimpleRenderer {
    materials: Component<SimpleMaterial>,

    surface: SurfaceHandle,
    shader: ShaderHandle,
    drawcalls: DrawCommandBuffer<DrawOrder>,

    global_ambient: Color<f32>,
    dir_lits: Vec<(String, String)>,
    point_lits: Vec<(String, String, String)>,
}

impl Drop for SimpleRenderer {
    fn drop(&mut self) {
        video::delete_surface(self.surface);
        video::delete_shader(self.shader);
    }
}

impl SimpleRenderer {
    /// Creates a new `SimpleRenderer`.
    pub fn new() -> Result<Self, Error> {
        // Create shader state.
        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 3)
            .with(Attribute::Normal, 3)
            .with_optional(Attribute::Texcoord0, 2)
            .finish();

        let mut uniforms = UniformVariableLayout::build()
            .with("u_ModelViewMatrix", UniformVariableType::Matrix4f)
            .with("u_MVPMatrix", UniformVariableType::Matrix4f)
            .with("u_ViewNormalMatrix", UniformVariableType::Matrix4f)
            .with("u_GlobalAmbient", UniformVariableType::Vector3f)
            .with("u_Diffuse", UniformVariableType::Vector3f)
            .with("u_DiffuseTexture", UniformVariableType::Texture)
            .with("u_Specular", UniformVariableType::Vector3f)
            .with("u_SpecularTexture", UniformVariableType::Texture)
            .with("u_Shininess", UniformVariableType::F32);

        let mut dir_lits = Vec::new();
        let mut point_lits = Vec::new();

        for i in 0..MAX_DIR_LITS {
            let name = (
                format!("u_DirLitViewDir[{0}]", i),
                format!("u_DirLitColor[{0}]", i),
            );

            uniforms = uniforms
                .with(name.0.as_str(), UniformVariableType::Vector3f)
                .with(name.1.as_str(), UniformVariableType::Vector3f);

            dir_lits.push(name);
        }

        for i in 0..MAX_POINT_LITS {
            let name = (
                format!("u_PointLitViewPos[{0}]", i),
                format!("u_PointLitColor[{0}]", i),
                format!("u_PointLitAttenuation[{0}]", i),
            );

            uniforms = uniforms
                .with(name.0.as_str(), UniformVariableType::Vector3f)
                .with(name.1.as_str(), UniformVariableType::Vector3f)
                .with(name.2.as_str(), UniformVariableType::Vector3f);

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
            include_str!("shaders/simple.vs")
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
            include_str!("shaders/simple.fs")
        );

        let shader = video::create_shader(params, vs, fs)?;

        let params = SurfaceParams::default();
        let surface = video::create_surface(params)?;

        Ok(SimpleRenderer {
            materials: Component::new(),
            surface: surface,
            shader: shader,
            drawcalls: DrawCommandBuffer::new(),
            dir_lits: dir_lits,
            point_lits: point_lits,
            global_ambient: Color::gray(),
        })
    }

    #[inline]
    pub fn add(&mut self, ent: Entity, material: SimpleMaterial) -> Option<SimpleMaterial> {
        self.materials.add(ent, material)
    }

    #[inline]
    pub fn has(&self, ent: Entity) -> bool {
        self.materials.has(ent)
    }

    #[inline]
    pub fn material(&self, ent: Entity) -> Option<&SimpleMaterial> {
        self.materials.get(ent)
    }

    #[inline]
    pub fn material_mut(&mut self, ent: Entity) -> Option<&mut SimpleMaterial> {
        self.materials.get_mut(ent)
    }

    #[inline]
    pub fn remove(&mut self, ent: Entity) {
        self.materials.remove(ent)
    }

    #[inline]
    pub fn set_global_ambient<T: Into<Color<f32>>>(&mut self, color: T) {
        self.global_ambient = color.into();
    }
}

impl super::Renderer for SimpleRenderer {
    type Mtl = SimpleMaterial;

    fn add_mtl(&mut self, ent: Entity, mtl: Self::Mtl) {
        self.add(ent, mtl);
    }

    fn mtl(&self, ent: Entity) -> Option<&Self::Mtl> {
        self.material(ent)
    }

    fn mtl_mut(&mut self, ent: Entity) -> Option<&mut Self::Mtl> {
        self.material_mut(ent)
    }

    fn remove_mtl(&mut self, ent: Entity) {
        self.remove(ent);
    }

    fn submit(&mut self, camera: &Camera, lits: &[Lit], meshes: &[MeshRenderer]) {
        use crayon::math::prelude::{InnerSpace, Matrix, MetricSpace, SquareMatrix};

        let view_matrix = camera.transform.view_matrix();
        let projection_matrix = camera.frustum().to_matrix();
        let mut lits = Vec::from(lits);

        for mesh in meshes {
            let model_matrix = mesh.transform.matrix();
            let mv = view_matrix * model_matrix;
            let mvp = projection_matrix * mv;
            let vn = mv.invert().and_then(|v| Some(v.transpose())).unwrap_or(mv);

            let mut dc = Draw::new(self.shader, mesh.mesh);
            dc.set_uniform_variable("u_ModelViewMatrix", mv);
            dc.set_uniform_variable("u_MVPMatrix", mvp);
            dc.set_uniform_variable("u_ViewNormalMatrix", vn);

            let mat = self.material(mesh.ent).cloned().unwrap_or_default();
            let diffuse = mat.diffuse_texture.unwrap_or(crate::default().white);
            let specular = mat.specular_texture.unwrap_or(crate::default().white);

            let mut ambient = mat.ambient.rgb();
            ambient[0] *= self.global_ambient.r;
            ambient[1] *= self.global_ambient.g;
            ambient[2] *= self.global_ambient.b;

            dc.set_uniform_variable("u_GlobalAmbient", ambient);
            dc.set_uniform_variable("u_Diffuse", mat.diffuse.rgb());
            dc.set_uniform_variable("u_DiffuseTexture", diffuse);
            dc.set_uniform_variable("u_Specular", mat.specular.rgb());
            dc.set_uniform_variable("u_SpecularTexture", specular);
            dc.set_uniform_variable("u_Shininess", mat.shininess);

            lits.sort_by_key(|v| mesh.transform.position.distance2(v.transform.position) as u32);

            let (mut dir_index, mut point_index) = (0, 0);
            for lit in &lits {
                match lit.source {
                    LitSource::Dir => {
                        if dir_index < self.dir_lits.len() {
                            let names = &self.dir_lits[dir_index];
                            let mut dir = view_matrix * lit.transform.forward().extend(0.0);
                            let mut color = lit.color.rgb();
                            color[0] *= lit.intensity;
                            color[1] *= lit.intensity;
                            color[2] *= lit.intensity;
                            dc.set_uniform_variable(&names.0, dir.truncate().normalize());
                            dc.set_uniform_variable(&names.1, color);
                            dir_index += 1;
                        }
                    }
                    LitSource::Point { radius, smoothness } => {
                        if point_index < self.point_lits.len() {
                            let names = &self.point_lits[point_index];
                            let mut pos = view_matrix * lit.transform.position.extend(1.0);
                            pos /= pos.w;
                            let attenuation = Vector3::new(
                                1.0,
                                -1.0 / (radius + smoothness * radius * radius),
                                -smoothness / (radius + smoothness * radius * radius),
                            );
                            let mut color = lit.color.rgb();
                            color[0] *= lit.intensity;
                            color[1] *= lit.intensity;
                            color[2] *= lit.intensity;
                            dc.set_uniform_variable(&names.0, pos.truncate());
                            dc.set_uniform_variable(&names.1, color);
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
        self.drawcalls.submit(surface).unwrap();
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
