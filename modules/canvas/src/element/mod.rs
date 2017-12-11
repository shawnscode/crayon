pub mod text;
pub mod image;

use crayon::math;
use crayon::ecs::VecArena;

use renderer::CanvasRenderer;
use assets::CanvasAssets;
use errors::*;

#[derive(Debug, Clone)]
pub enum Element {
    Empty(Empty),
    Text(text::Text),
    Image(image::Image),
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
    pub fn visible(&self) -> bool {
        match *self {
            Element::Empty(ref element) => element.visible,
            Element::Text(ref element) => element.visible,
            Element::Image(ref element) => element.visible,
        }
    }
}

impl Element {
    pub(crate) fn prefered_size(&self, assets: &mut CanvasAssets) -> Option<math::Vector2<f32>> {
        match *self {
            Element::Empty(_) => None,
            Element::Text(ref element) => element.prefered_size(&mut assets.fonts),
            Element::Image(ref element) => element.prefered_size(&mut assets.atlas),
        }
    }

    pub(crate) fn draw(&self,
                       renderer: &mut CanvasRenderer,
                       assets: &mut CanvasAssets,
                       size: math::Vector2<f32>)
                       -> Result<()> {
        match *self {
            Element::Empty(_) => Ok(()),
            Element::Text(ref element) => element.draw(renderer, &mut assets.fonts, size),
            Element::Image(ref element) => element.draw(renderer, &mut assets.atlas, size),
        }
    }
}

impl From<Empty> for Element {
    fn from(v: Empty) -> Self {
        Element::Empty(v)
    }
}

impl From<text::Text> for Element {
    fn from(v: text::Text) -> Self {
        Element::Text(v)
    }
}

impl From<image::Image> for Element {
    fn from(v: image::Image) -> Self {
        Element::Image(v)
    }
}