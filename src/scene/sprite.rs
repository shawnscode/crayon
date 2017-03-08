use graphics;
use resource;
use ecs::VecStorage;

/// A Sprite is a texture mapped planar mesh and associated material that
/// can be rendered in the world. In simpler terms, it's a quick and easy
/// way to draw 2D images.
#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    diffuse: graphics::Color,
    additive: graphics::Color,
    texture: Option<resource::ResourceHandle>,
}

declare_component!(Sprite, VecStorage);

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            diffuse: graphics::Color::white(),
            additive: graphics::Color::black(),
            texture: None,
        }
    }
}

impl Sprite {
    /// Return main color of `Sprite`.
    pub fn color(&self) -> graphics::Color {
        self.diffuse
    }

    /// Set the main color of `Sprite`.
    pub fn set_color(&mut self, color: &graphics::Color) {
        self.diffuse = *color;
    }

    /// Return main color of `Sprite`.
    pub fn additive_color(&self) -> graphics::Color {
        self.additive
    }

    /// Set the main color of `Sprite`.
    pub fn set_additive_color(&mut self, color: &graphics::Color) {
        self.additive = *color;
    }

    pub fn texture(&self) -> Option<resource::ResourceHandle> {
        self.texture
    }

    pub fn set_texture(&mut self, texture: Option<resource::ResourceHandle>) {
        self.texture = texture;
    }
}