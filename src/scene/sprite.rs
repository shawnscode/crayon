use graphics;
use resource;
use ecs::VecStorage;

/// A Sprite is a texture mapped planar mesh and associated material that
/// can be rendered in the world. In simpler terms, it's a quick and easy
/// way to draw 2D images.
#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    color: graphics::Color,
    texture: Option<resource::ResourceHandle>,
}

declare_component!(Sprite, VecStorage);

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            color: graphics::Color::white(),
            texture: None,
        }
    }
}

impl Sprite {
    /// Return main color of `Sprite`.
    pub fn color(&self) -> graphics::Color {
        self.color
    }

    /// Set the main color of `Sprite`.
    pub fn set_color(&mut self, color: &graphics::Color) {
        self.color = *color;
    }

    pub fn texture(&self) -> Option<resource::ResourceHandle> {
        self.texture
    }

    pub fn set_texture(&mut self, texture: Option<resource::ResourceHandle>) {
        self.texture = texture;
    }
}