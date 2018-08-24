use std::sync::Arc;

use crayon::errors::*;
use crayon::utils::HandlePool;

use assets::{PrefabHandle, WorldResourcesShared};
use renderers::{MeshRenderer, Renderable, Renderer};
use scene::SceneGraph;
use tags::Tags;

impl_handle!(Entity);

pub struct World<T: Renderer> {
    entities: HandlePool,
    pub tags: Tags,
    pub scene: SceneGraph,
    pub renderables: Renderable,
    pub renderer: T,
    pub res: Arc<WorldResourcesShared>,
}

impl<T: Renderer> World<T> {
    pub fn new(res: Arc<WorldResourcesShared>, renderer: T) -> Self {
        World {
            entities: HandlePool::new(),
            tags: Tags::new(),
            scene: SceneGraph::new(),
            renderables: Renderable::new(),
            renderer: renderer,
            res: res,
        }
    }

    /// Creates a new Entity.
    pub fn create(&mut self) -> Entity {
        world_impl::create(&mut self.entities, &mut self.scene)
    }

    /// Removes a Entity and all of its descendants from this world.
    pub fn remove(&mut self, ent: Entity) -> Option<Vec<Entity>> {
        world_impl::remove(
            &mut self.entities,
            &mut self.scene,
            &mut self.renderables,
            &mut self.tags,
            ent,
        )
    }

    /// Finds a Entity by name and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    #[inline]
    pub fn find<N: AsRef<str>>(&self, name: N) -> Option<Entity> {
        world_impl::find(&self.scene, &self.tags, name)
    }

    /// Instantiates a prefab into entities of this world.
    pub fn instantiate(&mut self, handle: PrefabHandle) -> Result<Entity> {
        if let Some(prefab) = self.res.prefab(handle) {
            world_impl::instantiate(
                &mut self.entities,
                &mut self.scene,
                &mut self.renderables,
                &mut self.tags,
                &prefab,
            )
        } else {
            bail!("{:?} is not valid.", handle);
        }
    }

    pub fn advance(&mut self) {
        self.renderables.draw(&mut self.renderer, &self.scene);
    }
}

pub mod world_impl {
    use super::*;
    use assets::Prefab;

    pub fn create(entities: &mut HandlePool, scene: &mut SceneGraph) -> Entity {
        let ent = entities.create().into();
        scene.add(ent);
        ent
    }

    pub fn remove(
        entities: &mut HandlePool,
        scene: &mut SceneGraph,
        renderables: &mut Renderable,
        tags: &mut Tags,
        ent: Entity,
    ) -> Option<Vec<Entity>> {
        if let Some(deletions) = scene.remove(ent) {
            for &v in &deletions {
                entities.free(v);
                tags.remove(v);
                renderables.remove_mesh(v);
                renderables.remove_lit(v);
                renderables.remove_camera(v);
            }

            Some(deletions)
        } else {
            None
        }
    }

    pub fn instantiate(
        mut entities: &mut HandlePool,
        mut scene: &mut SceneGraph,
        renderables: &mut Renderable,
        tags: &mut Tags,
        prefab: &Prefab,
    ) -> Result<Entity> {
        let mut root = None;
        let mut nodes = Vec::new();
        nodes.push((None, 0));

        while let Some((parent, idx)) = nodes.pop() {
            let n = &prefab.nodes[idx];
            let e = create(&mut entities, &mut scene);

            tags.add(e, &n.name);
            scene.set_local_transform(e, n.local_transform);

            if let Some(parent) = parent {
                scene.set_parent(e, parent, false).unwrap();
            }

            if let Some(mesh) = n.mesh_renderer {
                let mut mr = MeshRenderer::default();
                mr.mesh = prefab.meshes[mesh];
                renderables.add_mesh(e, mr);
            }

            if let Some(sib) = n.next_sib {
                nodes.push((parent, sib));
            }

            if let Some(child) = n.first_child {
                nodes.push((Some(e), child));
            }

            if root.is_none() {
                root = Some(e);
            }
        }

        return Ok(root.unwrap());
    }

    pub fn find<N: AsRef<str>>(scene: &SceneGraph, tags: &Tags, name: N) -> Option<Entity> {
        let mut components = name.as_ref().trim_left_matches('/').split('/');
        if let Some(first) = components.next() {
            for &v in &scene.roots {
                if let Some(n) = tags.name(v) {
                    if n == first {
                        let mut iter = v;
                        while let Some(component) = components.next() {
                            if component == "" {
                                continue;
                            }

                            let mut found = false;
                            for child in scene.children(iter) {
                                if let Some(n) = tags.name(child) {
                                    if n == component {
                                        iter = child;
                                        found = true;
                                        break;
                                    }
                                }
                            }

                            if !found {
                                return None;
                            }
                        }

                        while let Some(component) = components.next() {
                            if component == "" {
                                continue;
                            }

                            return None;
                        }

                        return Some(iter);
                    }
                }
            }
        }

        None
    }

}
