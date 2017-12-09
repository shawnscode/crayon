use crayon::ecs::HashMapArena;
use crayon::{math, utils};

use assets::{FontHandle, FontSystem};
use renderer::*;
use errors::*;

#[derive(Debug, Clone)]
pub struct Text {
    pub visible: bool,
    pub text: String,
    pub size: u32,
    pub color: utils::Color,
    pub font: Option<FontHandle>,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            text: "".to_owned(),
            size: 16,
            color: utils::Color::black(),
            font: None,
            visible: true,
        }
    }
}

declare_component!(Text, HashMapArena);

impl Text {
    pub fn draw(&self,
                renderer: &mut CanvasRenderer,
                fonts: &mut FontSystem,
                size: math::Vector2<f32>)
                -> Result<()> {
        let color = self.color.into();

        let h_wrap = if size.x <= 0.0 { None } else { Some(size.x) };
        let v_wrap = if size.y <= 0.0 { None } else { Some(size.y) };

        let (texture, glyphs) =
            fonts
                .layout(self.font, &self.text, self.size as f32, h_wrap, v_wrap)?;

        for (uv, screen) in glyphs {
            let min = math::Vector2::new(screen.min.x as f32, screen.min.y as f32);
            let max = math::Vector2::new(screen.max.x as f32, screen.max.y as f32);

            let verts = [CanvasVertex::new([min.x, -max.y], [uv.min.x, uv.max.y], color),
                         CanvasVertex::new([min.x, -min.y], [uv.min.x, uv.min.y], color),
                         CanvasVertex::new([max.x, -min.y], [uv.max.x, uv.min.y], color),
                         CanvasVertex::new([max.x, -max.y], [uv.max.x, uv.max.y], color)];

            let idxes = [0, 1, 2, 2, 3, 0];
            renderer.submit(&verts, &idxes, Some(texture))?;
        }

        Ok(())
    }

    pub fn prefered_size(&self, fonts: &mut FontSystem) -> Option<math::Vector2<f32>> {
        let (min, max) = fonts.bounding_box(self.font, &self.text, self.size as f32, None, None);
        Some((max.x - min.x, max.y - min.y).into())
    }
}