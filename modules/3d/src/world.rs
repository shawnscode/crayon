use std::sync::Arc;

use crayon::errors::*;
use crayon::utils::HandlePool;

use assets::{PrefabHandle, WorldResourcesShared};
use renderers::{MeshRenderer, RenderPipeline, Renderer};
use scene::SceneGraph;
use tags::Tags;

impl_handle!(Entity);

pub struct World<T: RenderPipeline> {
    entities: HandlePool,

    pub tags: Tags,
    pub scene: SceneGraph,
    pub renderer: Renderer,
    pub pipeline: T,
    pub res: Arc<WorldResourcesShared>,
}

impl<T: RenderPipeline> World<T> {
    pub fn new(res: Arc<WorldResourcesShared>, pipeline: T) -> Self {
        World {
            entities: HandlePool::new(),
            tags: Tags::new(),
            scene: SceneGraph::new(),
            renderer: Renderer::new(),
            pipeline: pipeline,
            res: res,
        }
    }

    /// Creates a new Entity.
    pub fn create(&mut self) -> Entity {
        let ent = self.entities.create().into();
        self.scene.add(ent);
        ent
    }

    /// Removes a Entity and all of its descendants from this world.
    pub fn remove(&mut self, ent: Entity) -> Option<Vec<Entity>> {
        if let Some(deletions) = self.scene.remove(ent) {
            for &v in &deletions {
                self.entities.free(v);
                self.tags.remove(v);
                self.renderer.remove_mesh(v);
                self.renderer.remove_lit(v);
                self.renderer.remove_camera(v);
            }

            Some(deletions)
        } else {
            None
        }
    }

    /// Finds a Entity by name and returns it.
    ///
    /// If no Entity with name can be found, None is returned. If name contains a '/' character,
    /// it traverses the hierarchy like a path name.
    #[inline]
    pub fn find<N: AsRef<str>>(&self, name: N) -> Option<Entity> {
        world_impl::find(&self.scene, &self.tags, name)
    }

    pub fn advance(&mut self) {
        self.renderer.draw(&mut self.pipeline, &self.scene);
    }

    /// Instantiates a prefab into entities of this world.
    pub fn instantiate(&mut self, handle: PrefabHandle) -> Result<Entity> {
        if let Some(prefab) = self.res.prefab(handle) {
            let mut root = None;
            let mut nodes = Vec::new();
            nodes.push((None, 0));

            while let Some((parent, idx)) = nodes.pop() {
                let n = &prefab.nodes[idx];
                let e = self.create();

                self.tags.add(e, &n.name);
                self.scene.set_local_transform(e, n.local_transform);

                if let Some(parent) = parent {
                    self.scene.set_parent(e, parent, false).unwrap();
                }

                if let Some(mesh) = n.mesh_renderer {
                    let mut mr = MeshRenderer::default();
                    mr.mesh = prefab.meshes[mesh];
                    self.renderer.add_mesh(e, mr);
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

        bail!("{:?} is not valid.", handle);
    }
}

pub mod world_impl {
    use super::*;

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
                                break;
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
