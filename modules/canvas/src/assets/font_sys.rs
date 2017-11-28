use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;

use crayon::{application, resource, utils, futures, graphics, math};
use crayon::futures::Future;
use rusttype;

use super::font::{Font, FontHandle};
use super::font_error::*;
use renderer::CanvasVertex;

pub struct FontSystem {
    fallback: Font,
    fonts: utils::ObjectPool<FontState>,
    handles: HashMap<utils::HashValue<Path>, utils::Handle>,

    texture_cache: rusttype::gpu_cache::Cache,
    texture: Option<graphics::TextureHandle>,

    resource: Arc<resource::ResourceSystemShared>,
    video: Arc<graphics::GraphicsSystemShared>,
}

impl FontSystem {
    pub fn new(ctx: &application::Context) -> Self {
        let fallback = include_bytes!("../../assets/fonts/FiraSans-Regular.ttf");

        FontSystem {
            fonts: utils::ObjectPool::new(),
            handles: HashMap::new(),
            fallback: Font::new(&fallback[..]),

            texture_cache: rusttype::gpu_cache::Cache::new(1024, 1024, 0.25, 0.25),
            texture: None,

            resource: ctx.shared::<resource::ResourceSystem>().clone(),
            video: ctx.shared::<graphics::GraphicsSystem>().clone(),
        }
    }

    pub fn load<P>(&mut self, path: P) -> FontHandle
        where P: AsRef<Path>
    {
        let hash = path.as_ref().into();
        if let Some(handle) = self.handles.get(&hash) {
            return (*handle).into();
        }

        let slave = FontSystemLoader {};
        let state = FontState::NotReady(self.resource.load(slave, path));

        let handle = self.fonts.create(state);
        self.handles.insert(hash, handle);
        handle.into()
    }

    pub fn unload<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        let hash = path.as_ref().into();
        if let Some(handle) = self.handles.remove(&hash) {
            self.fonts.free(handle);
        }
    }

    pub(crate) fn advance(&mut self) {
        let fonts: Vec<_> = self.fonts.iter().collect();
        for v in fonts {
            self.fonts.get_mut(v).map(|v| v.poll());
        }
    }

    #[inline]
    pub fn get(&self, handle: FontHandle) -> Option<&Font> {
        self.fonts.get(handle).and_then(|v| v.try_ready())
    }

    pub fn draw(&mut self,
                handle: Option<FontHandle>,
                text: &str,
                scale: f32)
                -> Result<(Vec<CanvasVertex>, Vec<u16>, graphics::TextureHandle)> {
        let (id, font) = if let Some(handle) = handle {
            if let Some(v) = self.fonts.get_mut(handle).and_then(|v| v.try_ready()) {
                ((handle.index() + 1) as usize, v)
            } else {
                (0, &self.fallback)
            }
        } else {
            (0, &self.fallback)
        };

        for v in font.layout(text, scale, None) {
            self.texture_cache.queue_glyph(id, v);
        }

        let handle = self.update_texture_cache()?;

        let mut verts = Vec::new();
        let mut idxes = Vec::new();
        let color = [0, 0, 0, 255];
        let dimensions = (640.0, 480.0);

        for v in font.layout(text, scale, None) {
            if let Ok(Some((uv, screen))) = self.texture_cache.rect_for(0, &v) {
                let min = (math::Vector2::new(screen.min.x as f32 / dimensions.0 - 0.5,
                                              1.0 - screen.min.y as f32 / dimensions.1 - 0.5)) *
                          2.0;
                let max = (math::Vector2::new(screen.max.x as f32 / dimensions.0 - 0.5,
                                              1.0 - screen.max.y as f32 / dimensions.1 - 0.5)) *
                          2.0;

                let offset = verts.len() as u16;
                verts.push(CanvasVertex::new([min.x, max.y], [uv.min.x, uv.max.y], color));
                verts.push(CanvasVertex::new([min.x, min.y], [uv.min.x, uv.min.y], color));
                verts.push(CanvasVertex::new([max.x, min.y], [uv.max.x, uv.min.y], color));
                verts.push(CanvasVertex::new([max.x, max.y], [uv.max.x, uv.max.y], color));

                idxes.push(offset + 0);
                idxes.push(offset + 1);
                idxes.push(offset + 2);
                idxes.push(offset + 2);
                idxes.push(offset + 3);
                idxes.push(offset + 0);
            }
        }

        Ok((verts, idxes, handle))
    }

    fn update_texture_cache(&mut self) -> Result<graphics::TextureHandle> {
        let handle = if self.texture.is_none() {
            let mut setup = graphics::TextureSetup::default();
            setup.filter = graphics::TextureFilter::Linear;
            setup.mipmap = false;
            setup.dimensions = (1024, 1024);
            setup.format = graphics::TextureFormat::U8;

            self.video.create_texture(setup, None)?
        } else {
            self.texture.unwrap()
        };

        let video = &self.video;
        self.texture_cache
            .cache_queued(|rect, data| {
                              let rect = graphics::Rect::new(math::Point2::new(rect.min.x as i32,
                                                                               rect.min.y as i32),
                                                             math::Point2::new(rect.max.x as i32,
                                                                               rect.max.y as i32));
                              video.update_texture(handle, rect, data);
                          })
            .unwrap();

        Ok(handle)
    }
}

enum FontState {
    Disposed,
    Ready(Font),
    NotReady(resource::ResourceFuture<Font, Error>),
}

impl FontState {
    fn poll(&mut self) {
        *self = match *self {
            FontState::Disposed => FontState::Disposed,
            FontState::Ready(_) => return,
            FontState::NotReady(ref mut future) => {
                match future.poll() {
                    Err(_) => FontState::Disposed,
                    Ok(futures::Async::NotReady) => return,
                    Ok(futures::Async::Ready(v)) => FontState::Ready(v),
                }
            }
        };
    }

    fn try_ready(&self) -> Option<&Font> {
        match self {
            &FontState::Ready(ref v) => Some(v),
            _ => None,
        }
    }
}

struct FontSystemLoader {}

impl resource::ResourceArenaLoader for FontSystemLoader {
    type Item = Font;
    type Error = Error;

    fn insert(&self, _: &Path, bytes: &[u8]) -> Result<Self::Item> {
        Ok(Font::new(bytes))
    }
}