use std::sync::Arc;
use std::collections::HashMap;

use application::Context;
use ecs::{ArenaMut, Component, Entity, Fetch, FetchMut, System, VecArena, World};
use graphics::{GraphicsSystem, GraphicsSystemShared, ShaderHandle, SurfaceHandle};
use utils::{HandleObjectPool, HashValue};

use scene::{Camera, Light, MeshRenderer, Node, Transform};
use scene::material::{Material, MaterialHandle, ShaderPair};
use scene::renderer::{RenderDataCollectTask, RenderTask, SceneUniformVariables};
use scene::errors::*;
use scene::factory;

pub struct Scene {
    world: World,
    materials: HandleObjectPool<Material>,
    video: Arc<GraphicsSystemShared>,
    fallback: Material,
    shader: ShaderHandle,
    shader_uniforms: HashMap<ShaderHandle, HashMap<SceneUniformVariables, HashValue<str>>>,
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
        let sso = video.shader(shader).unwrap();
        let fallback = Material::new(ShaderPair {
            handle: shader,
            sso: sso,
        });

        Ok(Scene {
            world: world,
            materials: materials,
            video: video,
            shader: shader,
            shader_uniforms: HashMap::new(),
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

    pub fn update_shader_uniforms<T1, T2>(&mut self, shader: ShaderHandle, pairs: T1) -> Result<()>
    where
        T1: Iterator<Item = (SceneUniformVariables, T2)>,
        T2: Into<HashValue<str>>,
    {
        if let Some(state) = self.video.shader(shader) {
            let mut uvs = self.shader_uniforms.get_mut(&shader).unwrap();

            for (suv, field) in pairs {
                let field = field.into();
                if let Some(tt) = state.uniform_variable(field) {
                    if tt == suv.into() {
                        uvs.insert(suv, field);
                    } else {
                        bail!(ErrorKind::UniformTypeInvalid);
                    }
                } else {
                    bail!(ErrorKind::UniformUndefined);
                }
            }

            Ok(())
        } else {
            bail!("Undefined shader handle.");
        }
    }

    #[inline(always)]
    pub fn create_material(&mut self, shader: ShaderHandle) -> Result<MaterialHandle> {
        if let Some(state) = self.video.shader(shader) {
            if !self.shader_uniforms.contains_key(&shader) {
                self.shader_uniforms.insert(shader, HashMap::new());
            }

            let m = self.materials.create(Material::new(ShaderPair {
                handle: shader,
                sso: state,
            }));

            Ok(m.into())
        } else {
            bail!("Undefined shader handle.");
        }
    }

    #[inline(always)]
    pub fn material(&self, handle: MaterialHandle) -> Option<&Material> {
        self.materials.get(*handle)
    }

    #[inline(always)]
    pub fn material_mut(&mut self, handle: MaterialHandle) -> Option<&mut Material> {
        self.materials.get_mut(*handle)
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
        let env = task.run_mut_at(&self.world)?;

        let task = RenderTask {
            video: &self.video,
            materials: &self.materials,
            surface: surface,
            fallback: &self.fallback,
            shader_binds: &self.shader_uniforms,
            view_matrix: view,
            projection_matrix: projection,
            data: env,
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
