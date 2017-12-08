use crayon::{math, utils};

use assets::AtlasSystem;
use renderer::*;
use errors::*;

#[derive(Debug, Clone, Copy)]
pub struct Image {
    pub visible: bool,
    pub color: utils::Color,
    // pub atlas: Option<AtlasHandle>,
}

impl Default for Image {
    fn default() -> Self {
        Image {
            visible: true,
            color: utils::Color::white(),
        }
    }
}

impl Image {
    pub fn prefered_size(&self, atlas: &mut AtlasSystem) -> Option<math::Vector2<f32>> {
        None
    }

    pub fn draw(&self,
                renderer: &mut CanvasRenderer,
                atlas: &mut AtlasSystem,
                size: math::Vector2<f32>)
                -> Result<()> {
        // let texture = self.atlas.map(|v| atlas.texture(v));

        let color = self.color.into();
        let verts = [CanvasVertex::new([0.0, 0.0], [0.0, 0.0], color),
                     CanvasVertex::new([size.x, 0.0], [0.0, 0.0], color),
                     CanvasVertex::new([size.x, size.y], [0.0, 0.0], color),
                     CanvasVertex::new([0.0, size.y], [0.0, 0.0], color)];

        let idxes = [0, 1, 2, 2, 3, 0];
        renderer.submit(&verts, &idxes, None)?;
        Ok(())
    }
}
