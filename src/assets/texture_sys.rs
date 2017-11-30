use std::path::Path;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;
use futures;

use resource;
use graphics;
use application;
use utils::HashValue;

use super::texture::Texture;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
    }
}

pub trait TextureFormat {
    fn parse(bytes: &[u8]) -> Result<Texture>;
}

pub struct TextureSystem {
    textures: Arc<RwLock<resource::arena::ArenaWithCache<Texture>>>,
    video_textures: Arc<RwLock<resource::arena::Arena<graphics::TextureHandle>>>,
    resource: Arc<resource::ResourceSystemShared>,
    video: Arc<graphics::GraphicsSystemShared>,
}

impl TextureSystem {
    pub fn new(ctx: &application::Context) -> Self {
        let resource = ctx.shared::<resource::ResourceSystem>().clone();
        let video = ctx.shared::<graphics::GraphicsSystem>().clone();

        let textures = resource::arena::ArenaWithCache::with_capacity(0);
        let video_textures = resource::arena::Arena::new();

        TextureSystem {
            textures: Arc::new(RwLock::new(textures)),
            video_textures: Arc::new(RwLock::new(video_textures)),
            resource: resource,
            video: video,
        }
    }

    pub fn load<T, P>(&self, path: P) -> resource::ResourceFuture<Arc<Texture>, Error>
        where P: AsRef<Path>,
              T: TextureFormat + Send + Sync + 'static
    {
        let slave = TextureSystemLoader::<T>::new(self.textures.clone());
        self.resource.load(slave, path)
    }

    pub fn load_into_video<T, P>(&self,
                                 path: P,
                                 setup: graphics::TextureSetup)
                                 -> resource::ResourceFuture<Arc<graphics::TextureHandle>, Error>
        where P: AsRef<Path>,
              T: TextureFormat + Send + Sync + 'static
    {
        let hash = path.as_ref().into();

        {
            let handles = self.video_textures.read().unwrap();
            if let Some(v) = handles.get(hash) {
                let (tx, rx) = futures::sync::oneshot::channel();
                tx.send(Ok(v.clone())).is_ok();
                return resource::ResourceFuture::new(rx);
            }
        }

        let slave = TextureSystemMapper {
            hash: hash,
            textures: self.video_textures.clone(),
            video: self.video.clone(),
            setup: setup,
        };

        let first_pass = {
            let slave = TextureSystemLoader::<T>::new(self.textures.clone());
            self.resource.load(slave, path)
        };

        self.resource.map(slave, first_pass)
    }

    pub(crate) fn unload_unused(&self) {
        let video = self.video.clone();
        let delete = |_, v: Arc<graphics::TextureHandle>| { video.delete_texture(*v); };

        self.textures.write().unwrap().unload_unused(None);
        self.video_textures
            .write()
            .unwrap()
            .unload_unused(Some(&delete));
    }
}

struct TextureSystemLoader<T>
    where T: TextureFormat
{
    textures: Arc<RwLock<resource::arena::ArenaWithCache<Texture>>>,
    _phantom: PhantomData<T>,
}

impl<T> TextureSystemLoader<T>
    where T: TextureFormat
{
    fn new(textures: Arc<RwLock<resource::arena::ArenaWithCache<Texture>>>) -> Self {
        TextureSystemLoader {
            textures: textures,
            _phantom: PhantomData,
        }
    }
}

impl<T> resource::ResourceArenaLoader for TextureSystemLoader<T>
    where T: TextureFormat + Send + Sync + 'static
{
    type Item = Arc<Texture>;
    type Error = Error;

    fn get(&self, path: &Path) -> Option<Self::Item> {
        let mut arena = self.textures.write().unwrap();
        arena.get(path).map(|v| v.clone())
    }

    fn insert(&self, path: &Path, bytes: &[u8]) -> Result<Self::Item> {
        let item = Arc::new(T::parse(bytes)?);

        {
            let mut arena = self.textures.write().unwrap();
            arena.insert(path, item.clone());
        }

        Ok(item)
    }
}

struct TextureSystemMapper {
    hash: HashValue<Path>,
    textures: Arc<RwLock<resource::arena::Arena<graphics::TextureHandle>>>,

    setup: graphics::TextureSetup,
    video: Arc<graphics::GraphicsSystemShared>,
}

impl resource::ResourceArenaMapper for TextureSystemMapper {
    type Source = Arc<Texture>;
    type Item = Arc<graphics::TextureHandle>;
    type Error = Error;

    fn map(&self, src: &Self::Source) -> Result<Self::Item> {
        {
            let textures = self.textures.write().unwrap();
            if let Some(v) = textures.get(self.hash) {
                return Ok(v.clone());
            }
        }

        let mut setup = self.setup;
        setup.dimensions = src.dimensions();
        setup.format = src.format();
        let handle = self.video.create_texture(setup, Some(src.data()))?;

        let rc = Arc::new(handle);
        self.textures.write().unwrap().insert(self.hash, rc.clone());
        return Ok(rc);
    }
}