use std::sync::Arc;

use application::Context;
use ecs::{ArenaMut, Component, Entity, Fetch, FetchMut, System, VecArena, World};
use graphics::{GraphicsSystem, GraphicsSystemShared, ShaderHandle, SurfaceHandle, UniformVariable};
use utils::{HandleObjectPool, HashValue};

use scene::{Camera, Light, MeshRenderer, Node, Transform};
use scene::material::{Material, MaterialHandle};
use scene::renderer::{RenderDataCollectTask, RenderTask};
use scene::errors::*;
use scene::factory;

pub struct Scene {
    world: World,
    materials: HandleObjectPool<Material>,
    video: Arc<GraphicsSystemShared>,
    fallback: Material,
    shader: ShaderHandle,
}

impl Drop for Scene {
    fn drop(&mut self) {
        self.video.delete_shader(self.shader);
    }
}

impl Scene {
    pub fn new(ctx: &Context) -> Result<Self> {
        let video = ctx.shared::<GraphicsSystem>().clone();

        let mut world = World::new();
        world.register::<Node>();
        world.register::<Transform>();
        world.register::<SceneNode>();

        let materials = HandleObjectPool::new();

        let shader = factory::shader::undefined(&video)?;
        let shader_state = video.shader_state(shader).unwrap();
        let fallback = Material::new(shader, shader_state);

        Ok(Scene {
            world: world,
            materials: materials,
            video: video,
            shader: shader,
            fallback: fallback,
        })
    }

    #[inline(always)]
    pub fn arena<T>(&self) -> Fetch<T>
    where
        T: Component,
    {
        self.world.arena::<T>()
    }

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

    #[inline(always)]
    pub fn create_material(&mut self, shader: ShaderHandle) -> Result<MaterialHandle> {
        if let Some(state) = self.video.shader_state(shader) {
            Ok(self.materials.create(Material::new(shader, state)).into())
        } else {
            bail!("Undefined shader handle.");
        }
    }

    #[inline(always)]
    pub fn update_material_uniform<T1, T2>(
        &mut self,
        handle: MaterialHandle,
        field: T1,
        variable: T2,
    ) -> Result<()>
    where
        T1: Into<HashValue<str>>,
        T2: Into<UniformVariable>,
    {
        if let Some(mat) = self.materials.get_mut(*handle) {
            mat.set_uniform_variable(field, variable)
        } else {
            bail!("Undefined material handle.");
        }
    }

    #[inline(always)]
    pub fn delete_material(&mut self, handle: MaterialHandle) -> Result<()> {
        if self.materials.free(handle).is_none() {
            bail!("Undefined material handle");
        }

        Ok(())
    }

    /// Renders objects into `Surface` from `Camera`.
    pub fn render(&mut self, surface: SurfaceHandle, camera: Entity) -> Result<()> {
        let (view, projection) = {
            if let Some(SceneNode::Camera(v)) = self.world.get::<SceneNode>(camera) {
                let tree = self.world.arena::<Node>();
                let arena = self.world.arena::<Transform>();
                let view = Transform::world_view_matrix(&tree, &arena, camera)?;
                let projection = v.matrix();
                (view, projection)
            } else {
                bail!(ErrorKind::NonCameraFound);
            }
        };

        let mut task = RenderDataCollectTask::new(view);
        task.run_mut_at(&self.world);

        let task = RenderTask {
            video: &self.video,
            materials: &self.materials,
            surface: surface,
            fallback: &self.fallback,
            view_matrix: view,
            projection_matrix: projection,
            data: task.data,
        };
        task.run_at(&self.world);

        Ok(())
    }
}


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
