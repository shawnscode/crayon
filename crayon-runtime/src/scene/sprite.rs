//! A texture mapped planar mesh.

use resource;
use ecs;
use ecs::VecStorage;
use math;

/// A Sprite is a texture mapped planar mesh and associated material that
/// can be rendered in the world. In simpler terms, it's a quick and easy
/// way to draw 2D images.
#[derive(Debug, Clone)]
pub struct Sprite {
    visible: bool,

    color: math::Color,
    additive: math::Color,

    texture: Option<resource::TexturePtr>,
    texture_rect: ((f32, f32), (f32, f32)),

    mat: Option<resource::MaterialPtr>,
}

declare_component!(Sprite, VecStorage);

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            visible: true,
            color: math::Color::white(),
            additive: math::Color::transparent(),
            texture: None,
            texture_rect: ((0f32, 0f32), (1f32, 1f32)),
            mat: None,
        }
    }
}

impl Sprite {
    /// Get main color of `Sprite`.
    pub fn color(&self) -> math::Color {
        self.color
    }

    /// Set the main color of `Sprite`.
    pub fn set_color(&mut self, color: &math::Color) {
        self.color = *color;
    }

    /// Get additive color of `Sprite`.
    pub fn additive_color(&self) -> math::Color {
        self.additive
    }

    /// Set the additive color of `Sprite`.
    pub fn set_additive_color(&mut self, color: &math::Color) {
        self.additive = *color;
    }

    /// Get the underlying texture of `Sprite`.
    pub fn texture(&self) -> Option<resource::TexturePtr> {
        self.texture.clone()
    }

    /// Set the underlying texture of `Sprite`.
    pub fn set_texture(&mut self, texture: Option<resource::TexturePtr>) {
        self.texture = texture;
    }

    /// Get the rectangle this sprite use on its texture.
    pub fn texture_rect(&self) -> ((f32, f32), (f32, f32)) {
        self.texture_rect
    }

    /// Set the rectangle this sprite use on its texture.
    pub fn set_texture_rect(&mut self, position: (f32, f32), size: (f32, f32)) {
        self.texture_rect = (position, size);
    }
}

impl super::Renderable for Sprite {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible
    }

    fn material(&self) -> Option<&resource::MaterialPtr> {
        self.mat.as_ref()
    }
}

impl Sprite {
    pub fn new(world: &mut ecs::World) -> ecs::Entity {
        use super::{Transform, Rect};
        world
            .build()
            .with_default::<Transform>()
            .with_default::<Rect>()
            .with_default::<Sprite>()
            .finish()
    }

    pub fn new_with_atlas_frame(world: &mut ecs::World,
                                frame: &resource::AtlasFrame)
                                -> ecs::Entity {
        use super::{Transform, Rect};
        let mut rect = Rect::default();
        rect.set_pivot(math::Vector2::new(frame.pivot.0, frame.pivot.1));

        let mut sprite = Sprite::default();
        sprite.set_texture_rect(frame.position, frame.size);
        sprite.set_texture(Some(frame.texture.clone()));

        world
            .build()
            .with_default::<Transform>()
            .with::<Rect>(rect)
            .with::<Sprite>(sprite)
            .finish()
    }
}