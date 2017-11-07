use std::path::Path;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::marker::PhantomData;

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
        Resource(resource::errors::Error, resource::errors::ErrorKind);
    }
}

pub trait TextureFormat {
    fn parse(bytes: &[u8]) -> Result<Texture>;
}

pub struct TextureSystem {
    textures: Arc<RwLock<resource::cache::ArenaWithCache<Texture>>>,
    video_textures: Arc<RwLock<HashMap<HashValue<Path>, Arc<graphics::TextureHandle>>>>,
    resource: Arc<resource::ResourceSystemShared>,
    video: Arc<graphics::GraphicsSystemShared>,
}

impl TextureSystem {
    pub fn new(ctx: &application::Context) -> Self {
        let resource = ctx.shared::<resource::ResourceSystem>().clone();
        let video = ctx.shared::<graphics::GraphicsSystem>().clone();

        let textures = resource::cache::ArenaWithCache::with_capacity(0);
        let video_textures = HashMap::new();

        TextureSystem {
            textures: Arc::new(RwLock::new(textures)),
            video_textures: Arc::new(RwLock::new(video_textures)),
            resource: resource,
            video: video,
        }
    }

    pub fn load<T, P>(&self, path: P) -> resource::ResourceFuture<Texture, Error>
        where P: AsRef<Path>,
              T: TextureFormat + Send + Sync + 'static
    {
        let slave = TextureSystemLoader::<T>::new(self.textures.clone());
        self.resource.load(slave, path)
    }

    pub fn load_into_video<T, P>(&self,
                                 path: P,
                                 setup: graphics::TextureSetup)
                                 -> resource::ResourceFuture<graphics::TextureHandle, Error>
        where P: AsRef<Path>,
              T: TextureFormat + Send + Sync + 'static
    {
        let slave = TextureSystemMapper {
            hash: path.as_ref().into(),
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
}

struct TextureSystemLoader<T>
    where T: TextureFormat
{
    textures: Arc<RwLock<resource::cache::ArenaWithCache<Texture>>>,
    _phantom: PhantomData<T>,
}

impl<T> TextureSystemLoader<T>
    where T: TextureFormat
{
    fn new(textures: Arc<RwLock<resource::cache::ArenaWithCache<Texture>>>) -> Self {
        TextureSystemLoader {
            textures: textures,
            _phantom: PhantomData,
        }
    }
}

impl<T> resource::ResourceArenaLoader for TextureSystemLoader<T>
    where T: TextureFormat + Send + Sync + 'static
{
    type Item = Texture;
    type Error = Error;

    fn get(&self, path: &Path) -> Option<Arc<Self::Item>> {
        let mut arena = self.textures.write().unwrap();
        arena.get(path)
    }

    fn parse(&self, bytes: &[u8]) -> Result<Self::Item> {
        T::parse(bytes)
    }

    fn insert(&self, path: &Path, item: Arc<Self::Item>) {
        let mut arena = self.textures.write().unwrap();
        arena.insert(path, item);
    }
}

struct TextureSystemMapper {
    hash: HashValue<Path>,
    textures: Arc<RwLock<HashMap<HashValue<Path>, Arc<graphics::TextureHandle>>>>,

    setup: graphics::TextureSetup,
    video: Arc<graphics::GraphicsSystemShared>,
}

impl resource::ResourceArenaMapper for TextureSystemMapper {
    type Source = Texture;
    type Item = graphics::TextureHandle;
    type Error = Error;

    fn map(&self, src: &Self::Source) -> Result<Arc<Self::Item>> {
        let mut setup = self.setup;
        setup.dimensions = src.dimensions();
        setup.format = src.format();

        let handle = self.video.create_texture(setup, Some(src.data())).unwrap();

        let rc = Arc::new(handle);
        self.textures.write().unwrap().insert(self.hash, rc.clone());

        return Ok(rc);
    }
}