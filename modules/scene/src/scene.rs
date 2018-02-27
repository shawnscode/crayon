use std::sync::Arc;

use crayon::application::Context;
use crayon::ecs::prelude::*;
use crayon::graphics::prelude::*;
use crayon::graphics::assets::prelude::*;
use crayon::resource::utils::prelude::*;
use crayon::utils::{HandleObjectPool, HashValue};

use node::Node;
use transform::Transform;
use element::Element;
use renderer::Renderer;

use assets::prelude::*;
use assets::material::Material;
use errors::*;

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
/// scene, a list of drawcalls will be generated and submitted to `GraphicsSystem`.
///
/// ```rust,ignore
/// let _mesh_node = scene.create_node(MeshRenderer { ... });
/// let _lit_node = scene.create_node(Lit { ... });
/// let camera = Camera::perspective(math::Deg(60.0), 6.4 / 4.8, 0.1, 1000.0);
/// let camera_node = scene.create_node(camera);
/// self.scene.render(surface, camera_node)?;
/// ```
///
pub struct Scene {
    pub(crate) world: World,

    pub(crate) video: Arc<GraphicsSystemShared>,
    pub(crate) materials: HandleObjectPool<Material>,
    pub(crate) pipelines: Registery<PipelineParams>,

    pub(crate) renderer: Renderer,
    pub(crate) fallback: Option<MaterialHandle>,
}

impl Scene {
    /// Creates a new `Scene`.
    pub fn new(ctx: &Context) -> Result<Self> {
        let video = ctx.shared::<GraphicsSystem>();

        let mut world = World::new();
        world.register::<Node>();
        world.register::<Transform>();
        world.register::<Element>();

        let materials = HandleObjectPool::new();
        let scene = Scene {
            world: world,
            video: video.clone(),

            pipelines: Registery::new(),
            materials: materials,
            fallback: None,

            renderer: Renderer::new(ctx)?,
        };

        Ok(scene)
    }

    /// Immutably borrows the arena of component. The borrow lasts until the returned
    /// `Fetch` exits scope. Multiple immutable borrows can be taken out at the same
    /// time.
    ///
    /// # Panics
    ///
    /// - Panics if user has not register the arena with type `T`.
    /// - Panics if the value is currently mutably borrowed.
    #[inline]
    pub fn arena<T>(&self) -> Fetch<T>
    where
        T: Component,
    {
        self.world.arena::<T>()
    }

    /// Mutably borrows the wrapped arena. The borrow lasts until the returned
    /// `FetchMut` exits scope. The value cannot be borrowed while this borrow
    /// is active.
    ///
    /// # Panics
    ///
    /// - Panics if user has not register the arena with type `T`.
    /// - Panics if the value is currently borrowed.
    #[inline]
    pub fn arena_mut<T>(&self) -> FetchMut<T>
    where
        T: Component,
    {
        self.world.arena_mut::<T>()
    }

    /// Creates a `Element`.
    pub fn create_node<T1>(&mut self, node: T1) -> Entity
    where
        T1: Into<Element>,
    {
        self.world
            .build()
            .with_default::<Node>()
            .with_default::<Transform>()
            .with(node.into())
            .finish()
    }

    /// Updates a `Element`.
    pub fn update_node<T1>(&mut self, handle: Entity, node: T1) -> Result<()>
    where
        T1: Into<Element>,
    {
        if !self.world.is_alive(handle) {
            return Err(Error::NonNodeFound);
        }

        unsafe {
            *self.world.arena_mut::<Element>().get_unchecked_mut(handle) = node.into();
            Ok(())
        }
    }

    /// Deletes a node and its descendants from the `Scene`.
    pub fn delete_node(&mut self, handle: Entity) -> Result<()> {
        let descendants: Vec<_> = Node::descendants(&self.arena::<Node>(), handle).collect();
        for v in descendants {
            self.world.free(v);
        }

        Node::remove_from_parent(&mut self.arena_mut::<Node>(), handle)?;
        self.world.free(handle);

        Ok(())
    }

    /// Lookups pipeline object from location.
    pub fn lookup_pipeline(&self, location: Location) -> Option<PipelineHandle> {
        self.pipelines.lookup(location).map(|v| v.into())
    }

    /// Creates a new pipeline object that indicates the whole render pipeline of `Scene`.
    pub fn create_pipeline(&mut self, setup: PipelineSetup) -> Result<PipelineHandle> {
        if let Some(handle) = self.lookup_pipeline(setup.location()) {
            self.pipelines.inc_rc(*handle);
            return Ok(handle.into());
        }

        let (location, setup, links) = setup.into();
        let params = setup.params.clone();
        let shader = self.video.create_shader(setup)?;

        Ok(self.pipelines
            .create(location, PipelineParams::new(shader, params, links))
            .into())
    }

    /// Creates a new material instance from shader.
    pub fn create_material(&mut self, pipeline: PipelineHandle) -> Result<MaterialHandle> {
        if self.pipelines.get(*pipeline).is_some() {
            let m = self.materials.create(Material::new(pipeline));
            Ok(m.into())
        } else {
            Err(Error::PipelineHandleInvalid(pipeline))
        }
    }

    /// Updates the uniform variable of material.
    pub fn update_material<T1, T2>(&mut self, h: MaterialHandle, f: T1, v: T2) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        if let Some(m) = self.materials.get_mut(*h) {
            if let Some(pipeline) = self.pipelines.get(*m.pipeline) {
                m.set_uniform_variable(pipeline, f, v)?;
            }

            Ok(())
        } else {
            Err(Error::MaterialHandleInvalid(h))
        }
    }

    /// Deletes the material instance from `Scene`. Any meshes that associated with a
    /// invalid/deleted material handle will be drawed with a fallback material marked
    /// with purple color.
    #[inline]
    pub fn delete_material(&mut self, handle: MaterialHandle) -> Result<()> {
        if self.materials.free(handle).is_none() {
            Err(Error::MaterialHandleInvalid(handle))
        } else {
            Ok(())
        }
    }

    pub fn advance(&mut self, camera: Entity) -> Result<()> {
        self.renderer.advance(&self.world, camera)
    }

    /// Draws the underlaying depth buffer of shadow mapping pass. This is used for
    /// debugging.
    pub fn draw_shadow(&mut self, surface: SurfaceHandle) -> Result<()> {
        self.renderer.draw_shadow(surface)
    }

    /// Renders objects into `Surface` from `Camera`.
    pub fn draw(&mut self, surface: SurfaceHandle, camera: Entity) -> Result<()> {
        if self.fallback.is_none() {
            let undefined = factory::pipeline::undefined(self)?;
            self.fallback = Some(self.create_material(undefined)?);
        }

        self.renderer.draw(self, surface, camera)?;
        Ok(())
    }
}
