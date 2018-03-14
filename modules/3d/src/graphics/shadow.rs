use std::collections::HashMap;

use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::graphics::prelude::*;
use crayon::graphics::assets::prelude::*;

use graphics::DrawOrder;
use graphics::graph::{RenderShadowCaster, SimpleRenderGraph};

use assets::factory;
use components::prelude::*;
use errors::*;

#[derive(Debug, Clone, Copy)]
struct ShadowSurface {
    render_texture: RenderTextureHandle,
    surface: SurfaceHandle,
}

/// A shadow mapping builder.
///
/// Some techniques that used to avoid artifacts could be found at :
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ee416324(v=vs.85).aspx
///
pub struct RenderShadow {
    video: GraphicsSystemGuard,
    depth_shader: ShaderHandle,
    draw_shader: ShaderHandle,

    shadow_casters: HashMap<Entity, (ShadowSurface, RenderShadowCaster)>,
    shadow_surfaces: Vec<ShadowSurface>,
}

impl RenderShadow {
    /// Craetes a new `RenderShadow`.
    pub fn new(ctx: &Context) -> Result<Self> {
        let mut video = GraphicsSystemGuard::new(ctx.shared::<GraphicsSystem>().clone());

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

    /// Advances one frame, builds the depth buffer of shadow mapping technique.
    pub fn advance(&mut self, world: &World, graph: &SimpleRenderGraph) -> Result<()> {
        for (_, v) in self.shadow_casters.drain() {
            self.shadow_surfaces.push(v.0);
        }

        for lit in graph.lits() {
            if let Some(caster) = lit.shadow {
                let surface = self.alloc_surface()?;
                self.shadow_casters.insert(lit.handle, (surface, caster));
            }
        }

        GenerateRenderShadow { shadow: self }.run_at(world)
    }

    /// Gets the handle of depth buffer.
    pub fn depth_render_texture(&self, caster: Entity) -> Option<RenderTextureHandle> {
        if let Some(&(ss, _)) = self.shadow_casters.get(&caster) {
            Some(ss.render_texture)
        } else {
            None
        }
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

    fn alloc_surface(&mut self) -> Result<ShadowSurface> {
        if let Some(surface) = self.shadow_surfaces.pop() {
            return Ok(surface);
        }

        let mut setup = RenderTextureSetup::default();
        setup.format = RenderTextureFormat::Depth16;
        // FIXME: The dimensions of shadow texture should be configuratable.
        setup.dimensions = (256, 256);
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

struct GenerateRenderShadow<'a> {
    shadow: &'a RenderShadow,
}

impl<'a, 'b> System<'a> for GenerateRenderShadow<'b> {
    type ViewWith = (
        Fetch<'a, Node>,
        Fetch<'a, Transform>,
        Fetch<'a, MeshRenderer>,
    );
    type Result = Result<()>;

    fn run(&mut self, view: View, data: Self::ViewWith) -> Self::Result {
        unsafe {
            for handle in view {
                let mesh = data.2.get_unchecked(handle);

                if !mesh.visible || !mesh.shadow_caster {
                    continue;
                }

                // Gets the underlying mesh params.
                let mso = if let Some(mso) = self.shadow.video.mesh(mesh.mesh) {
                    mso
                } else {
                    continue;
                };

                for (_, &(ss, rsc)) in &self.shadow.shadow_casters {
                    let m = Transform::world_matrix(&data.0, &data.1, handle)?;
                    let mvp = rsc.shadow_space_matrix * m;

                    let mut dc = DrawCall::new(self.shadow.depth_shader, mesh.mesh);
                    dc.set_uniform_variable("u_MVPMatrix", mvp);

                    for i in 0..mso.sub_mesh_offsets.len() {
                        let sdc = dc.build_sub_mesh(i)?;
                        self.shadow.video.submit(ss.surface, 0u64, sdc)?;
                    }
                }
            }
        }

        Ok(())
    }
}
