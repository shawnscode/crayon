pub mod text;

use crayon::{math, application};
use crayon::ecs::VecArena;

use renderer::CanvasRenderer;

#[derive(Debug, Clone)]
pub enum Element {
    Empty(Empty),
    Text(text::Text),
}

declare_component!(Element, VecArena);

#[derive(Debug, Clone, Copy)]
pub struct Empty {
    pub visible: bool,
}

impl Default for Element {
    fn default() -> Self {
        Element::Empty(Empty { visible: true })
    }
}

impl Element {
    pub fn draw(&self, ctx: &application::Context) {
        match *self {
            Element::Empty(_) => {}
            Element::Text(ref element) => element.draw(ctx),
        }
    }

    pub fn visible(&self) -> bool {
        match *self {
            Element::Empty(ref element) => element.visible,
            Element::Text(ref element) => element.visible,
        }
    }

    pub fn prefered_size(&self, ctx: &application::Context) -> Option<math::Vector2<f32>> {
        match *self {
            Element::Empty(_) => None,
            Element::Text(ref element) => element.prefered_size(ctx),
        }
    }
}