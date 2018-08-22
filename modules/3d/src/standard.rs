use std::sync::Arc;

use crayon::ecs::prelude::*;
use crayon::errors::*;

use assets::{PrefabHandle, WorldResourcesShared};
use renderer::{MeshRenderer, RenderPipeline, Renderer};
use scene::SceneGraph;

pub struct Standard<T: RenderPipeline> {
    pub world: World,
    pub scene: SceneGraph,
    pub renderer: Renderer,
    pub pipeline: T,
    pub res: Arc<WorldResourcesShared>,
}

impl<T: RenderPipeline> Standard<T> {
    pub fn new(res: Arc<WorldResourcesShared>, pipeline: T) -> Self {
        Standard {
            world: World::new(),
            scene: SceneGraph::new(),
            renderer: Renderer::new(),
            pipeline: pipeline,
            res: res,
        }
    }

    pub fn create(&mut self) -> Entity {
        let ent = self.world.create();
        self.scene.add(ent);
        ent
    }

    pub fn instantiate(&mut self, handle: PrefabHandle) -> Result<Entity> {
        if let Some(prefab) = self.res.prefab(handle) {
            let mut root = None;
            let mut nodes = Vec::new();
            nodes.push((None, 0));

            while let Some((parent, idx)) = nodes.pop() {
                let n = &prefab.nodes[idx];
                let e = self.create();

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

    pub fn advance(&mut self) {
        self.renderer.draw(&mut self.pipeline, &self.scene);
    }
}
