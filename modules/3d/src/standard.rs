use crayon::ecs::prelude::*;

use renderer::{RenderPipeline, Renderer};
use scene::SceneGraph;

pub struct Standard<T: RenderPipeline> {
    pub world: World,
    pub scene: SceneGraph,
    pub renderer: Renderer,
    pub pipeline: T,
}

impl<T: RenderPipeline> Standard<T> {
    pub fn new(pipeline: T) -> Self {
        Standard {
            world: World::new(),
            scene: SceneGraph::new(),
            renderer: Renderer::new(),
            pipeline: pipeline,
        }
    }

    pub fn create(&mut self) -> Entity {
        let ent = self.world.create();
        self.scene.add(ent);
        ent
    }

    pub fn advance(&mut self) {
        self.renderer.draw(&mut self.pipeline, &self.scene);
    }
}
