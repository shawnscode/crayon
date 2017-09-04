use core::application;
use ecs;
use graphics;

use super::errors::*;
use super::*;

/// The `Scene` system represents the abstract space in which the 2d widgets is laid
/// out and rendered.
pub struct Scene {
    world: ecs::World,
    camera: Option<ecs::Entity>,
    renderer: Renderer,

    ambient: graphics::Color,
    ambient_intensity: f32,
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
               ambient: graphics::Color::white(),
               ambient_intensity: 1.0,
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

    /// Set the ambient color of this scene.
    pub fn set_ambient_color(&mut self, color: graphics::Color) {
        self.ambient = color;
    }

    /// Get the ambient color.
    pub fn ambient_color(&mut self) -> graphics::Color {
        self.ambient
    }

    /// Set the ambient intensity of this scene.
    pub fn set_ambient_intensity(&mut self, intensity: f32) {
        self.ambient_intensity = intensity;
    }

    /// Get the ambient intensity.
    pub fn ambient_intensity(&self) -> f32 {
        self.ambient_intensity
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