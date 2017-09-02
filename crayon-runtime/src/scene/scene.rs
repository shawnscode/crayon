use core::application;
use ecs;

use super::errors::*;
use super::*;

/// The `Scene` system represents the abstract space in which the 2d widgets is laid
/// out and rendered.
pub struct Scene {
    world: ecs::World,
    camera: Option<ecs::Entity>,
    renderer: Renderer,
}

impl Scene {
    pub fn new(mut application: &mut application::Application) -> Result<Self> {
        let mut world = ecs::World::new();
        world.register::<Transform>();
        world.register::<Rect>();
        world.register::<Sprite>();
        world.register::<Camera>();
        world.register::<Mesh>();

        Ok(Scene {
               world: world,
               camera: None,
               renderer: Renderer::new(&mut application)?,
           })
    }

    /// Get `ecs::World`.
    pub fn world(&self) -> &ecs::World {
        &self.world
    }

    /// Get mutable `ecs::World`.
    pub fn world_mut(&mut self) -> &mut ecs::World {
        &mut self.world
    }

    /// Get the main camera.
    pub fn main_camera(&self) -> Option<ecs::Entity> {
        self.camera
    }

    /// Set the main camera.
    pub fn set_main_camera(&mut self, camera: ecs::Entity) {
        self.camera = Some(camera)
    }

    pub fn run_one_frame(&mut self, mut application: &mut application::Application) -> Result<()> {
        self.renderer.draw(&mut application, &self.world)?;
        Ok(())
    }
}

impl Scene {
    /// Create a empty sprite.
    pub fn sprite(world: &mut ecs::World) -> ecs::Entity {
        world
            .build()
            .with_default::<Transform>()
            .with_default::<Rect>()
            .with_default::<Sprite>()
            .finish()
    }

    pub fn camera(world: &mut ecs::World) -> ecs::Entity {
        world
            .build()
            .with_default::<Transform>()
            .with_default::<Camera>()
            .finish()
    }
}