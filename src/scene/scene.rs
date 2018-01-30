use std::sync::Arc;
use std::collections::HashMap;

use application::Context;
use ecs::{ArenaMut, Component, Entity, Fetch, FetchMut, VecArena, World};
use graphics::{GraphicsSystem, GraphicsSystemShared, ShaderHandle, ShaderStateObject,
               SurfaceHandle};
use utils::{HandleObjectPool, HashValue};

use scene::{Camera, Light, MeshRenderer, Node, Transform};
use scene::material::{Material, MaterialHandle};
use scene::renderer::{RenderGraph, RenderUniform};
use scene::errors::*;
use scene::factory;

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
/// And besides the spatial representation, `SceneNode` is used to provide graphical data
/// that could be used to render on the screen. A `SceneNode` could be one of `Camera`
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
    world: World,
    video: Arc<GraphicsSystemShared>,

    materials: HandleObjectPool<Material>,
    render_shaders: HashMap<ShaderHandle, Arc<RenderShader>>,
    render_env: RenderGraph,
    fallback: Option<MaterialHandle>,
}

impl Scene {
    /// Creates a new `Scene`.
    pub fn new(ctx: &Context) -> Result<Self> {
        let video = ctx.shared::<GraphicsSystem>();

        let mut world = World::new();
        world.register::<Node>();
        world.register::<Transform>();
        world.register::<SceneNode>();

        let materials = HandleObjectPool::new();
        let mut scene = Scene {
            world: world,
            materials: materials,
            fallback: None,
            video: video.clone(),

            render_shaders: HashMap::new(),
            render_env: RenderGraph::new(ctx)?,
        };

        factory::shader::setup(&mut scene, &video)?;
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
    #[inline(always)]
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
    #[inline(always)]
    pub fn arena_mut<T>(&self) -> FetchMut<T>
    where
        T: Component,
    {
        self.world.arena_mut::<T>()
    }

    #[inline(always)]
    pub fn create_node<T1>(&mut self, node: T1) -> Entity
    where
        T1: Into<SceneNode>,
    {
        self.world
            .build()
            .with_default::<Node>()
            .with_default::<Transform>()
            .with(node.into())
            .finish()
    }

    #[inline(always)]
    pub fn update_node<T1>(&mut self, handle: Entity, node: T1) -> Result<()>
    where
        T1: Into<SceneNode>,
    {
        if !self.world.is_alive(handle) {
            bail!(ErrorKind::HandleInvalid);
        }

        unsafe {
            *self.world
                .arena_mut::<SceneNode>()
                .get_unchecked_mut(handle) = node.into();
            Ok(())
        }
    }

    #[inline(always)]
    pub fn delete_node(&mut self, handle: Entity) -> Result<()> {
        Node::remove_from_parent(&mut self.arena_mut::<Node>(), handle)?;
        self.world.free(handle);
        Ok(())
    }

    /// Creates a new material instance from shader.
    #[inline(always)]
    pub fn create_material(&mut self, shader: ShaderHandle) -> Result<MaterialHandle> {
        if let Some(render_shader) = self.render_shaders.get(&shader) {
            let m = self.materials.create(Material::new(render_shader.clone()));
            Ok(m.into())
        } else {
            bail!(
                "Shader can NOT be used before initialization with `Scene::setup_render_shader`."
            );
        }
    }

    /// Gets a reference to the material.
    #[inline(always)]
    pub fn material(&self, handle: MaterialHandle) -> Option<&Material> {
        self.materials.get(*handle)
    }

    /// Gets a mutable reference to the material.
    #[inline(always)]
    pub fn material_mut(&mut self, handle: MaterialHandle) -> Option<&mut Material> {
        self.materials.get_mut(*handle)
    }

    /// Deletes the material instance from `Scene`. Any meshes that associated with a
    /// invalid/deleted material handle will be drawed with a fallback material marked
    /// with purple color.
    #[inline(always)]
    pub fn delete_material(&mut self, handle: MaterialHandle) -> Result<()> {
        if self.materials.free(handle).is_none() {
            bail!("Undefined material handle");
        }

        Ok(())
    }

    /// Renders objects into `Surface` from `Camera`.
    pub fn render(&mut self, surface: SurfaceHandle, camera: Entity) -> Result<()> {
        if self.fallback.is_none() {
            let undefined = factory::shader::undefined(&self.video)?;
            self.fallback = Some(self.create_material(undefined)?);
        }

        let fallback = self.materials.get(self.fallback.unwrap()).unwrap();
        self.render_env
            .render(&self.world, &self.materials, &fallback, surface, camera)?;

        Ok(())
    }

    /// `Shader`s that used in `Scene` could be filled with some build-in uniform variables
    /// for convenient, such likes `scn_ModelMatrix`, `scn_MVPMatrix` etc.. You can find the
    /// complete supported uniforms and corresponding information in enumeration
    /// `RenderUniform`.
    ///
    /// But you can also choose your own field definition for custom shader.
    ///
    /// ```rust,ignore
    /// scene.setup_render_shader(custom_shader, [
    ///     (RenderUniform::ModelMatrix, "u_ModelMatrix"),
    ///     (RenderUniform::ViewNormalMatrix, "u_NormalViewMatrix"),
    /// ]).unwrap();
    /// ```
    pub fn setup_render_shader<T1>(
        &mut self,
        shader: ShaderHandle,
        pairs: &[(RenderUniform, T1)],
    ) -> Result<()>
    where
        T1: Into<HashValue<str>> + Copy,
    {
        if self.render_shaders.get(&shader).is_some() {
            bail!("Duplicated initialization of `RenderShader`.");
        }

        if let Some(sso) = self.video.shader(shader) {
            let mut render_uniforms = HashMap::new();

            for &(uniform, field) in pairs {
                let field = field.into();
                if let Some(tt) = sso.uniform_variable(field) {
                    if tt == uniform.into() {
                        render_uniforms.insert(uniform, field);
                    } else {
                        bail!(ErrorKind::UniformTypeInvalid);
                    }
                } else {
                    bail!(ErrorKind::UniformUndefined);
                }
            }

            let rs = RenderShader {
                sso: sso,
                render_uniforms: render_uniforms,
                handle: shader,
            };

            self.render_shaders.insert(shader, Arc::new(rs));
            Ok(())
        } else {
            bail!("Undefined shader handle.");
        }
    }
}

///
#[derive(Debug, Clone)]
pub struct RenderShader {
    pub sso: Arc<ShaderStateObject>,
    pub render_uniforms: HashMap<RenderUniform, HashValue<str>>,
    pub handle: ShaderHandle,
}

/// The contrainer of components that supported in `Scene`.
#[derive(Debug, Clone, Copy)]
pub enum SceneNode {
    None,
    Light(Light),
    Camera(Camera),
    Mesh(MeshRenderer),
}

impl Component for SceneNode {
    type Arena = VecArena<SceneNode>;
}

impl Into<SceneNode> for Light {
    fn into(self) -> SceneNode {
        SceneNode::Light(self)
    }
}

impl Into<SceneNode> for Camera {
    fn into(self) -> SceneNode {
        SceneNode::Camera(self)
    }
}

impl Into<SceneNode> for MeshRenderer {
    fn into(self) -> SceneNode {
        SceneNode::Mesh(self)
    }
}

impl Into<SceneNode> for () {
    fn into(self) -> SceneNode {
        SceneNode::None
    }
}
