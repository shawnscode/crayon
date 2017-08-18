use graphics;
use resource;
use ecs::VecStorage;

/// A Sprite is a texture mapped planar mesh and associated material that
/// can be rendered in the world. In simpler terms, it's a quick and easy
/// way to draw 2D images.
#[derive(Debug, Clone)]
pub struct Sprite {
    visible: bool,
    color: graphics::Color,
    additive: graphics::Color,
    texture: Option<resource::TextureItem>,
}

declare_component!(Sprite, VecStorage);

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            visible: true,
            color: graphics::Color::white(),
            additive: graphics::Color::black(),
            texture: None,
        }
    }
}

impl Sprite {
    /// Get main color of `Sprite`.
    pub fn color(&self) -> graphics::Color {
        self.color
    }

    /// Set the main color of `Sprite`.
    pub fn set_color(&mut self, color: &graphics::Color) {
        self.color = *color;
    }

    /// Get additive color of `Sprite`.
    pub fn additive_color(&self) -> graphics::Color {
        self.additive
    }

    /// Set the additive color of `Sprite`.
    pub fn set_additive_color(&mut self, color: &graphics::Color) {
        self.additive = *color;
    }

    /// Get the underlying texture of `Sprite`.
    pub fn texture(&self) -> Option<&resource::TextureItem> {
        self.texture.as_ref()
    }

    /// Set the underlying texture of `Sprite`.
    pub fn set_texture(&mut self, texture: Option<resource::TextureItem>) {
        self.texture = texture;
    }
}

impl super::Renderable for Sprite {
    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible
    }
}