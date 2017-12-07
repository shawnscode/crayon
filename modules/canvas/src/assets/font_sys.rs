use std::path::Path;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use crayon::{application, resource, utils, graphics, math};
use rusttype;

use super::font::{Font, FontHandle, LayoutIter};
use super::errors::*;

pub struct FontSystem {
    dpi_factor: f32,
    texture_cache: FontTextureCache,
    cache: FontCache,
}

impl FontSystem {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        Ok(FontSystem {
               dpi_factor: 1.0,
               texture_cache: FontTextureCache::new(ctx),
               cache: FontCache::new(ctx)?,
           })
    }

    #[inline(always)]
    pub fn create_from(&mut self, location: resource::Location) -> Result<FontHandle> {
        self.cache.create_from(location)
    }

    #[inline(always)]
    pub fn lookup_from(&mut self, location: resource::Location) -> Option<FontHandle> {
        self.cache.lookup_from(location)
    }

    #[inline(always)]
    pub fn delete(&mut self, handle: FontHandle) {
        self.cache.delete(handle);
    }

    pub fn set_dpi_factor(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
    }

    /// The conservative pixel-boundary bounding box for this text. This is the smallest
    /// rectangle aligned to pixel boundaries that encloses the shape.
    pub fn bounding_box(&mut self,
                        handle: Option<FontHandle>,
                        text: &str,
                        scale: f32,
                        h_wrap: Option<f32>,
                        v_wrap: Option<f32>)
                        -> (math::Vector2<f32>, math::Vector2<f32>) {
        let (_, font) = self.cache.get(handle);
        font.bounding_box(text, scale, h_wrap, v_wrap)
    }

    /// A convenience function for laying out glyphs for a text.
    pub fn layout<'a, 'b>(&'a mut self,
                          handle: Option<FontHandle>,
                          text: &'b str,
                          scale: f32,
                          h_wrap_limit: Option<f32>,
                          v_wrap_limit: Option<f32>)
                          -> Result<(graphics::TextureHandle, FontGlyphIter<'a, 'b>)> {
        let (id, font) = self.cache.get(handle);

        let dpi_factor = self.dpi_factor;
        let h_wrap_limit = h_wrap_limit.map(|v| v * dpi_factor);
        let v_wrap_limit = v_wrap_limit.map(|v| v * dpi_factor);

        for v in font.layout(text, scale * dpi_factor, h_wrap_limit, v_wrap_limit) {
            self.texture_cache.add(id, v);
        }

        let handle = self.texture_cache.update_texture()?;

        Ok((handle,
            FontGlyphIter {
                texture_cache: &self.texture_cache,
                id: id,
                iter: font.layout(text, scale * dpi_factor, h_wrap_limit, v_wrap_limit),
                inverse_dpi_factor: 1.0 / dpi_factor,
            }))
    }
}

pub struct FontGlyphIter<'a, 'b> {
    texture_cache: &'a FontTextureCache,
    id: usize,
    iter: LayoutIter<'a, 'b>,
    inverse_dpi_factor: f32,
}

impl<'a, 'b> Iterator for FontGlyphIter<'a, 'b> {
    type Item = (rusttype::Rect<f32>, rusttype::Rect<i32>);

    fn next(&mut self) -> Option<Self::Item> {
        for v in &mut self.iter {
            if let Some((uv, mut screen)) = self.texture_cache.rect_for(self.id, &v) {
                screen.min.x = (screen.min.x as f32 * self.inverse_dpi_factor) as i32;
                screen.min.y = (screen.min.y as f32 * self.inverse_dpi_factor) as i32;
                screen.max.x = (screen.max.x as f32 * self.inverse_dpi_factor) as i32;
                screen.max.y = (screen.max.y as f32 * self.inverse_dpi_factor) as i32;
                return Some((uv, screen));
            }
        }

        None
    }
}

enum FontState {
    NotReady,
    Ready(Option<Font>),
    Err(String),
}

struct FontCache {
    fallback: Font,
    fonts: HashMap<FontHandle, Font>,
    states: resource::Registery<Arc<RwLock<FontState>>>,
    resource: Arc<resource::ResourceSystemShared>,
}

impl FontCache {
    fn new(ctx: &application::Context) -> Result<Self> {
        let fallback = include_bytes!("../../resources/fonts/FiraSans-Regular.ttf");

        let cache = FontCache {
            fallback: Font::new(&fallback[..]),
            fonts: HashMap::new(),
            states: resource::Registery::new(),
            resource: ctx.shared::<resource::ResourceSystem>().clone(),
        };

        Ok(cache)
    }

    fn lookup_from(&mut self, location: resource::Location) -> Option<FontHandle> {
        self.states.lookup(location).map(|v| v.into())
    }

    fn create_from(&mut self, location: resource::Location) -> Result<FontHandle> {
        if let Some(handle) = self.states.lookup(location) {
            self.states.inc_rc(handle);
            return Ok(handle.into());
        }

        let state = Arc::new(RwLock::new(FontState::NotReady));

        let loader = FontLoader { state: state.clone() };
        self.resource.load_async(loader, location.uri());

        Ok(self.states.create(location, state).into())
    }

    fn delete(&mut self, handle: FontHandle) {
        if self.states.dec_rc(handle.into()).is_some() {
            self.fonts.remove(&handle);
        }
    }

    fn get(&mut self, handle: Option<FontHandle>) -> (usize, &Font) {
        if let Some(handle) = handle {
            if let Some(state) = self.states.get(handle.into()) {
                let mut state = state.write().unwrap();
                let ready = {
                    match *state {
                        FontState::Ready(_) => true,
                        _ => false,
                    }
                };

                if ready {
                    let mut v = FontState::Ready(None);
                    ::std::mem::swap(&mut state as &mut FontState, &mut v);

                    match v {
                        FontState::Ready(Some(font)) => {
                            self.fonts.insert(handle, font);
                        }
                        _ => {}
                    }

                    return (handle.index() as usize + 1, self.fonts.get(&handle).unwrap());
                }
            }
        }

        (0, &self.fallback)
    }
}

struct FontTextureCache {
    texture_cache: rusttype::gpu_cache::Cache<'static>,
    texture: Option<graphics::TextureHandle>,
    video: Arc<graphics::GraphicsSystemShared>,
}

impl FontTextureCache {
    fn new(ctx: &application::Context) -> Self {
        let video = ctx.shared::<graphics::GraphicsSystem>().clone();

        FontTextureCache {
            texture_cache: rusttype::gpu_cache::Cache::new(1024, 1024, 0.25, 0.25),
            texture: None,
            video: video,
        }
    }

    #[inline]
    fn add(&mut self, id: usize, glyph: rusttype::PositionedGlyph) {
        self.texture_cache.queue_glyph(id, glyph.standalone());
    }

    #[inline]
    fn rect_for(&self,
                id: usize,
                glyph: &rusttype::PositionedGlyph)
                -> Option<(rusttype::Rect<f32>, rusttype::Rect<i32>)> {
        self.texture_cache.rect_for(id, glyph).unwrap()
    }

    fn update_texture(&mut self) -> Result<graphics::TextureHandle> {
        if self.texture.is_none() {
            let mut setup = graphics::TextureSetup::default();
            setup.filter = graphics::TextureFilter::Linear;
            setup.mipmap = false;
            setup.dimensions = (1024, 1024);
            setup.format = graphics::TextureFormat::U8;
            self.texture = Some(self.video.create_texture(setup, None)?);
        }

        let handle = self.texture.unwrap();
        let video = &self.video;
        self.texture_cache
            .cache_queued(|rect, data| {
                              let rect = utils::Rect::new(math::Point2::new(rect.min.x as i32,
                                                                            rect.min.y as i32),
                                                          math::Point2::new(rect.max.x as i32,
                                                                            rect.max.y as i32));
                              video.update_texture(handle, rect, data).unwrap();
                          })
            .unwrap();

        Ok(handle)
    }
}

impl Drop for FontTextureCache {
    fn drop(&mut self) {
        if let Some(handle) = self.texture.take() {
            self.video.delete_texture(handle);
        }
    }
}

struct FontLoader {
    state: Arc<RwLock<FontState>>,
}

impl resource::ResourceAsyncLoader for FontLoader {
    fn on_finished(&mut self, path: &Path, result: resource::errors::Result<&[u8]>) {
        let state = match result {
            Ok(bytes) => FontState::Ready(Some(Font::new(bytes))),
            Err(error) => {
                let error = format!("Failed to load font from {:?}, due to {:?}.", path, error);
                FontState::Err(error)
            }
        };

        *self.state.write().unwrap() = state;
    }
}