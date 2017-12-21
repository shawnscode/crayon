use super::errors::*;
use camera;
use crayon::{application, ecs, graphics};
use crayon::ecs::System;

use node;
use transform;

/// The `Scene` system represents the abstract space in which the 2d widgets is laid
/// out and rendered.
pub struct Scene {
    world: ecs::World,
    camera: CameraSystem,
    render: RenderSystem,
}

impl Scene {
    pub fn new() -> Result<Self> {
        let mut world = ecs::World::new();
        world.register::<node::Node>();
        world.register::<transform::Transform>();
        world.register::<camera::Camera>();

        Ok(Scene {
               world: world,
               camera: CameraSystem { main: None },
               render: RenderSystem {},
           })
    }

    pub fn update(&mut self, _: &application::Context) -> Result<()> {
        Ok(())
    }

    pub fn render(&mut self, ctx: &application::Context) -> Result<()> {
        self.camera.run_at(&self.world);

        if let Some(camera) = self.camera.main {
            self.render.run_at(&self.world);
        }

        Ok(())
    }
}

type RenderData<'a> = (ecs::Fetch<'a, transform::Transform>,
                       ecs::Fetch<'a, node::Node>,
                       ecs::Fetch<'a, camera::Camera>);

struct RenderSystem {}

impl<'a> ecs::System<'a> for RenderSystem {
    type ViewWith = RenderData<'a>;

    fn run(&mut self, view: ecs::View, _: Self::ViewWith) {
        for v in view {
            println!("draw {:?}", v);
        }
    }
}

struct CameraSystem {
    main: Option<ecs::Entity>,
}

impl<'a> ecs::System<'a> for CameraSystem {
    type ViewWith = RenderData<'a>;

    fn run(&mut self, view: ecs::View, _: Self::ViewWith) {
        if self.main.is_none() {
            for v in view {
                self.main = Some(v);
                return;
            }
        }
    }
}