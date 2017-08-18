use core::application;
use ecs;

use super::errors::*;
use super::{Camera, Rect, Transform, Sprite, SpriteRenderer};

/// The `Scene2d` system represents the abstract space in which the 2d widgets is laid
/// out and rendered.
pub struct Scene2d {
    world: ecs::World,
    camera: Option<ecs::Entity>,
    sprite_renderer: SpriteRenderer,
}

impl Scene2d {
    pub fn new(mut application: &mut application::Application) -> Result<Self> {
        let mut world = ecs::World::new();
        world.register::<Transform>();
        world.register::<Rect>();
        world.register::<Sprite>();
        world.register::<Camera>();

        Ok(Scene2d {
               world: world,
               camera: None,
               sprite_renderer: SpriteRenderer::new(&mut application)?,
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
        // We can't render the sprites without a camera.
        if let Some(camera) = self.camera {
            self.sprite_renderer
                .render(&mut application, &self.world, camera)?;
        }

        Ok(())
    }
}

impl Scene2d {
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