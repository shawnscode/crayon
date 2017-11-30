pub mod text;

use crayon::math;
use crayon::ecs::VecArena;

use renderer::CanvasRenderer;
use assets::FontSystem;
use errors::*;

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
    pub fn draw(&self,
                renderer: &mut CanvasRenderer,
                fonts: &mut FontSystem,
                size: math::Vector2<f32>)
                -> Result<()> {
        match *self {
            Element::Empty(_) => Ok(()),
            Element::Text(ref element) => element.draw(renderer, fonts, size),
        }
    }

    pub fn visible(&self) -> bool {
        match *self {
            Element::Empty(ref element) => element.visible,
            Element::Text(ref element) => element.visible,
        }
    }

    pub fn prefered_size(&self, fonts: &mut FontSystem) -> Option<math::Vector2<f32>> {
        match *self {
            Element::Empty(_) => None,
            Element::Text(ref element) => element.prefered_size(fonts),
        }
    }
}