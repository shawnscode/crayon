use std::collections::HashMap;

use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::math;
use crayon::rayon::prelude::*;
use crayon::video::assets::prelude::*;
use crayon::video::prelude::*;

use graphics::data::{RenderData, RenderShadowCaster};
use graphics::{DrawOrder, DrawSetup};

use assets::factory;
use components::prelude::*;
use errors::*;

/// A shadow mapping builder.
///
/// Some techniques that used to avoid artifacts could be found at :
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ee416324(v=vs.85).aspx
///
pub struct RenderShadow {
    video: VideoSystemGuard,
    depth_shader: ShaderHandle,
    draw_shader: ShaderHandle,
    shadow_casters: HashMap<Entity, (ShadowSurface, RenderShadowCaster)>,
    shadow_surfaces: Vec<ShadowSurface>,
}

impl RenderShadow {
    /// Craetes a new `RenderShadow`.
    pub fn new(ctx: &Context) -> Result<Self> {
        let mut video = VideoSystemGuard::new(ctx.video.clone());

        let shader = {
            let attributes = AttributeLayout::build()
                .with(Attribute::Position, 3)
                .finish();

            let uniforms = UniformVariableLayout::build()
                .with("u_MVPMatrix", UniformVariableType::Matrix4f)
                .finish();

            let mut setup = ShaderSetup::default();
            setup.vs = include_str!("../../assets/shadow.vs").to_owned();
            setup.fs = include_str!("../../assets/shadow.fs").to_owned();

            setup.params.attributes = attributes;
            setup.params.uniforms = uniforms;
            setup.params.render_state.depth_write = true;
            setup.params.render_state.depth_test = Comparison::Less;
            setup.params.render_state.cull_face = CullFace::Back;
            video.create_shader(setup)?
        };

        let draw_shader = {
            let attributes = AttributeLayout::build()
                .with(Attribute::Position, 3)
                .finish();

            let uniforms = UniformVariableLayout::build()
                .with("u_ShadowTexture", UniformVariableType::RenderTexture)
                .finish();

            let mut setup = ShaderSetup::default();
            setup.vs = include_str!("../../assets/shadow_texture.vs").to_owned();
            setup.fs = include_str!("../../assets/shadow_texture.fs").to_owned();

            setup.params.attributes = attributes;
            setup.params.uniforms = uniforms;
            video.create_shader(setup)?
        };

        Ok(RenderShadow {
            video: video,

            depth_shader: shader,
            draw_shader: draw_shader,

            shadow_casters: HashMap::new(),
            shadow_surfaces: Vec::new(),
        })
    }

    /// Gets the handle of depth buffer.
    pub fn depth_render_texture(&self, caster: Entity) -> Option<RenderTextureHandle> {
        if let Some(&(ss, _)) = self.shadow_casters.get(&caster) {
            Some(ss.render_texture)
        } else {
            None
        }
    }

    /// Advances one frame, builds the depth buffer of shadow mapping technique.
    pub fn build(&mut self, world: &World, data: &RenderData, setup: DrawSetup) -> Result<()> {
        for (_, v) in self.shadow_casters.drain() {
            self.shadow_surfaces.push(v.0);
        }

        for lit in &data.lits {
            if let Some(caster) = lit.shadow {
                let surface = self.alloc_surface(setup.max_shadow_resolution)?;
                self.shadow_casters.insert(lit.handle, (surface, caster));
            }
        }

        TaskGenShadow {
            data: data,
            shadow: self,
        }.run_with(world)
    }

    /// Draw the underlying depth buffer into the `surface`.
    pub fn draw(&self, caster: Entity, surface: SurfaceHandle) -> Result<()> {
        if let Some(render_texture) = self.depth_render_texture(caster) {
            let mesh = factory::mesh::quad(&self.video)?;

            let mut dc = DrawCall::new(self.draw_shader, mesh);
            dc.set_uniform_variable("u_ShadowTexture", render_texture);
            let sdc = dc.build_sub_mesh(0)?;
            self.video.submit(surface, 0u64, sdc)?;
        }

        Ok(())
    }

    fn alloc_surface(&mut self, resolution: math::Vector2<u32>) -> Result<ShadowSurface> {
        if let Some(surface) = self.shadow_surfaces.pop() {
            return Ok(surface);
        }

        let mut setup = RenderTextureSetup::default();
        setup.format = RenderTextureFormat::Depth16;
        setup.dimensions = resolution;
        let render_texture = self.video.create_render_texture(setup)?;

        let mut setup = SurfaceSetup::default();
        setup.set_attachments(&[], render_texture)?;
        setup.set_clear(None, 1.0, None);
        setup.set_order(DrawOrder::Shadow as u64);
        let surface = self.video.create_surface(setup)?;

        Ok(ShadowSurface {
            surface: surface,
            render_texture: render_texture,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct ShadowSurface {
    render_texture: RenderTextureHandle,
    surface: SurfaceHandle,
}

struct TaskGenShadow<'a> {
    data: &'a RenderData,
    shadow: &'a RenderShadow,
}

impl<'a, 'b> System<'a> for TaskGenShadow<'b> {
    type Data = (
        Fetch<'a, Node>,
        Fetch<'a, Transform>,
        Fetch<'a, MeshRenderer>,
    );
    type Err = Error;

    fn run(&mut self, entities: Entities, data: Self::Data) -> Result<()> {
        let video = &self.shadow.video;
        let world_transforms = &self.data.world_transforms;
        let shader = self.shadow.depth_shader;

        self.shadow
            .shadow_casters
            .par_iter()
            .for_each(|(_, &(ss, shadow))| {
                (entities, &data.0, &data.1, &data.2)
                    .par_join(&entities, 128)
                    .for_each(|(v, _, _, mesh)| {
                        if !mesh.visible || !mesh.shadow_caster {
                            return;
                        }

                        // Gets the underlying mesh params.
                        let mso = if let Some(mso) = video.mesh(mesh.mesh) {
                            mso
                        } else {
                            return;
                        };

                        // // Checks if mesh is visible for shadow frustum.
                        let m = world_transforms[&v].matrix();
                        let aabb = mso.aabb.transform(&(shadow.shadow_view_matrix * m));
                        if shadow.shadow_frustum.contains(&aabb) == math::PlaneRelation::Out {
                            return;
                        }

                        let mvp = shadow.shadow_space_matrix * m;
                        let mut dc = DrawCall::new(shader, mesh.mesh);
                        dc.set_uniform_variable("u_MVPMatrix", mvp);

                        let sdc = dc.build(MeshIndex::All).unwrap();
                        video.submit(ss.surface, 0u64, sdc).unwrap();
                    });
            });

        Ok(())
    }
}
