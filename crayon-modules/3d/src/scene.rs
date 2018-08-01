use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::resource::utils::prelude::*;
use crayon::video::prelude::*;

use assets::prelude::*;
use components::prelude::*;
use graphics::prelude::*;

use ent::{EntRef, EntRefMut};
use errors::*;
use resources::Resources;

#[derive(Debug, Clone, Copy, Default)]
pub struct SceneSetup {
    draw: DrawSetup,
}

/// `Scene`s contain the environments of your game. Its relative easy to think of each
/// unique scene as a unique level. In each `Scene`, you place your envrionments,
/// obstacles, and decorations, essentially designing and building your game in pieces.
///
/// The `Scene` is arranged with simple tree hierarchy. A tree `Node` may have many children
/// but only a single parent, with the effect of a parent applied to all its child nodes.
/// A spatial `Transform` is associated with every tree node, the world transformation
/// could be calculated by concatenating such `Transform`s along the ancestors.
/// ```rust,ignore
/// let mut tree = scene.arena_mut::<Node>();
/// let mut transforms = scene.arena_mut::<Transform>();
/// Node::set_parent(&mut tree, child, parent)?;
/// Transform::set_world_position(&tree, &mut transforms, child, [1.0, 0.0, 0.0])?;
/// ```
///
/// And besides the spatial representation, `Element` is used to provide graphical data
/// that could be used to render on the screen. A `Element` could be one of `Camera`
/// `Lit` or `MeshRenderer`. Everytime you call the `Scene::render` with proper defined
/// scene, a list of drawcalls will be generated and submitted to `VideoSystem`.
///
/// ```rust,ignore
/// let _mesh_node = scene.build(MeshRenderer { ... });
/// let _lit_node = scene.build(Lit { ... });
/// let camera = Camera::perspective(math::Deg(60.0), 6.4 / 4.8, 0.1, 1000.0);
/// let camera_node = scene.build(camera);
/// self.scene.render(surface, camera_node)?;
/// ```
///
pub struct Scene {
    pub resources: Resources,

    pub(crate) world: World,
    pub(crate) renderer: Renderer,
}

impl Scene {
    /// Creates a new `Scene`.
    pub fn new(ctx: &Context, setup: SceneSetup) -> Result<Self> {
        let mut world = World::new();
        world.register::<Node>();
        world.register::<Transform>();

        world.register::<Camera>();
        world.register::<Light>();
        world.register::<MeshRenderer>();

        let mut resources = Resources::new(ctx);

        let scene = Scene {
            world: world,
            renderer: Renderer::new(ctx, &mut resources, setup.draw)?,
            resources: resources,
        };

        Ok(scene)
    }

    /// Build a new `Entity` in this scene.
    pub fn create(&mut self) -> Entity {
        self.world
            .build()
            .with_default::<Node>()
            .with_default::<Transform>()
            .finish()
    }

    /// Gets the reference to entity.
    pub fn get(&self, id: Entity) -> Option<EntRef> {
        if self.world.is_alive(id) {
            Some(EntRef::new(&self.world, id))
        } else {
            None
        }
    }

    /// Gets the mutable reference to entity.
    pub fn get_mut(&mut self, id: Entity) -> Option<EntRefMut> {
        if self.world.is_alive(id) {
            Some(EntRefMut::new(&mut self.world, id))
        } else {
            None
        }
    }

    /// Deletes a node and its descendants from the `Scene`.
    pub fn delete(&mut self, handle: Entity) -> Result<()> {
        let descendants: Vec<_> = {
            let (_, mut nodes) = self.world.view_w1::<Node>();
            Node::descendants(&nodes, handle).collect()
        };

        for v in descendants {
            self.world.free(v);
        }

        {
            let (_, mut nodes) = self.world.view_w1::<Node>();
            Node::remove_from_parent(&mut nodes, handle)?;
        }

        self.world.free(handle);
        Ok(())
    }

    /// Advance to next frame.
    pub fn advance(&mut self, camera: Entity) -> Result<()> {
        self.renderer.advance(&self.world, camera)?;
        Ok(())
    }

    /// Draws the underlaying depth buffer of shadow mapping pass. This is used for
    /// debugging.
    pub fn draw_shadow<T>(&mut self, surface: T) -> Result<()>
    where
        T: Into<Option<SurfaceHandle>>,
    {
        self.renderer.draw_shadow(surface.into())
    }

    /// Renders objects into `Surface` from `Camera`.
    pub fn draw(&self, _: Entity) -> Result<()> {
        self.renderer.draw(&self.world, &self.resources)?;
        Ok(())
    }
}

impl Scene {
    /// Lookups pipeline object from location.
    #[inline]
    pub fn lookup_pipeline(&self, location: Location) -> Option<PipelineHandle> {
        self.resources.lookup_pipeline(location)
    }

    /// Creates a new pipeline object that indicates the whole render pipeline of `Scene`.
    #[inline]
    pub fn create_pipeline(&mut self, setup: PipelineSetup) -> Result<PipelineHandle> {
        self.resources.create_pipeline(setup)
    }

    /// Deletes a pipelie object.
    #[inline]
    pub fn delete_pipeline(&mut self, handle: PipelineHandle) {
        self.resources.delete_pipeline(handle)
    }

    /// Creates a new material instance from shader.
    #[inline]
    pub fn create_material(&mut self, setup: MaterialSetup) -> Result<MaterialHandle> {
        self.resources.create_material(setup)
    }

    /// Gets the reference to material.
    #[inline]
    pub fn material(&self, handle: MaterialHandle) -> Option<&Material> {
        self.resources.material(handle)
    }

    /// Gets the mutable reference to material.
    #[inline]
    pub fn material_mut(&mut self, handle: MaterialHandle) -> Option<&mut Material> {
        self.resources.material_mut(handle)
    }

    /// Deletes the material instance from `Scene`. Any meshes that associated with a
    /// invalid/deleted material handle will be drawed with a fallback material marked
    /// with purple color.
    #[inline]
    pub fn delete_material(&mut self, handle: MaterialHandle) {
        self.resources.delete_material(handle)
    }
}
