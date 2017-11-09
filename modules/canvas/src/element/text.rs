use crayon::ecs::HashMapArena;
use crayon::{graphics, application, math};

use assets::FontHandle;
use renderer::*;

/// An anchor aligns text horizontally to its given x position.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HorizontalAnchor {
    /// Anchor the left edge of the text
    Left,
    /// Anchor the horizontal mid-point of the text
    Center,
    /// Anchor the right edge of the text
    Right,
}

/// An anchor aligns text vertically to its given y position.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VerticalAnchor {
    /// Anchor the top edge of the text
    Top,
    /// Anchor the vertical mid-point of the text
    Center,
    /// Anchor the bottom edge of the text
    Bottom,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub visible: bool,
    pub text: String,
    pub size: u32,
    pub color: graphics::Color,
    pub horiontal: HorizontalAnchor,
    pub vertical: VerticalAnchor,
    pub font: Option<FontHandle>,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            text: "".to_owned(),
            size: 16,
            color: graphics::Color::black(),
            horiontal: HorizontalAnchor::Center,
            vertical: VerticalAnchor::Center,
            font: None,
            visible: true,
        }
    }
}

declare_component!(Text, HashMapArena);

impl Text {
    pub fn draw(&self, renderer: &mut CanvasRenderer) {
        renderer.draw_text("Hello, World! 你好，世界！");
    }

    pub fn prefered_size(&self, ctx: &application::Context) -> Option<math::Vector2<f32>> {
        None
    }
}