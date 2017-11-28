use std::str::Chars;

use crayon::math;
use rusttype;
use unicode_normalization::{UnicodeNormalization, Recompositions};

pub struct Font {
    font: rusttype::Font<'static>,
}

impl_handle!(FontHandle);

impl Font {
    pub fn new(bytes: &[u8]) -> Self {
        let collection = rusttype::FontCollection::from_bytes(bytes.to_vec());
        let font = collection.into_font().unwrap();
        Font { font: font }
    }
}

impl Font {
    /// The conservative pixel-boundary bounding box for this text. This is the smallest
    /// rectangle aligned to pixel boundaries that encloses the shape.
    pub fn bounding_box<'a, 'b>(&'a self,
                                text: &'b str,
                                scale: f32,
                                wrap_limits: Option<u32>)
                                -> (math::Vector2<f32>, math::Vector2<f32>) {
        // use std::cmp;
        let (min, max) = (math::Vector2::new(0.0, 0.0), math::Vector2::new(0.0, 0.0));
        for glyph in self.layout(text, scale, wrap_limits) {
            if let Some(_) = glyph.pixel_bounding_box() {
                // min.x = cmp::min(min.x, v.min.x);
                // min.y = cmp::min(min.y, v.min.y);

                // max.x = cmp::max(max.x, v.max.x);
                // max.y = cmp::max(max.y, v.max.y);
            }
        }

        (min, max)
    }

    /// A convenience function for laying out glyphs for a text.
    pub fn layout<'a, 'b>(&'a self,
                          text: &'b str,
                          scale: f32,
                          wrap_limits: Option<u32>)
                          -> LayoutIter<'a, 'b> {
        let scale = rusttype::Scale::uniform(scale);
        let v_metrics = self.font.v_metrics(scale);
        LayoutIter {
            font: &self.font,
            chars: text.nfc(),
            caret: rusttype::point(0.0, v_metrics.ascent),
            scale: scale,
            wrap_limits: wrap_limits,
            last_glyph: None,
        }
    }
}

#[derive(Clone)]
pub struct LayoutIter<'a, 'b> {
    font: &'a rusttype::Font<'a>,
    chars: Recompositions<Chars<'b>>,
    caret: rusttype::Point<f32>,
    scale: rusttype::Scale,
    wrap_limits: Option<u32>,
    last_glyph: Option<rusttype::GlyphId>,
}

impl<'a, 'b> Iterator for LayoutIter<'a, 'b> {
    type Item = rusttype::PositionedGlyph<'a>;

    fn next(&mut self) -> Option<rusttype::PositionedGlyph<'a>> {
        let v_metrics = self.font.v_metrics(self.scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        for c in &mut self.chars {
            if c.is_control() {
                match c {
                    '\r' => {
                        self.caret = rusttype::point(0.0, self.caret.y + advance_height);
                    }
                    '\n' => {}
                    _ => {}
                }
                continue;
            }

            let glyph = if let Some(v) = self.font.glyph(c) {
                v
            } else {
                continue;
            };

            if let Some(id) = self.last_glyph.take() {
                self.caret.x += self.font.pair_kerning(self.scale, id, glyph.id());
            }

            self.last_glyph = Some(glyph.id());
            let mut glyph = glyph.scaled(self.scale).positioned(self.caret);

            if let Some(width) = self.wrap_limits {
                if let Some(bb) = glyph.pixel_bounding_box() {
                    if bb.max.x > width as i32 {
                        self.caret = rusttype::point(0.0, self.caret.y + advance_height);
                        glyph = glyph.into_unpositioned().positioned(self.caret);
                        self.last_glyph = None;
                    }
                }
            }

            self.caret.x += glyph.unpositioned().h_metrics().advance_width;
            return Some(glyph);
        }

        None
    }
}