//! Build-in assets including `Texture`, `Atlas`, `Shader` etc..

mod bytes;
mod texture;

pub use self::bytes::Bytes;
pub use self::texture::Texture;

use std::sync::{Arc, RwLock};
use std::path::Path;
use std::collections::HashMap;

use resource;
use graphics;
use utils::HashValue;

pub struct GraphicsResourceSystem<T> {
    video: Arc<RwLock<graphics::GraphicsSystemShared>>,
    arena: HashMap<HashValue<Path>, Arc<T>>,
}

impl<T> GraphicsResourceSystem<T> {
    pub fn new(video: Arc<RwLock<graphics::GraphicsSystemShared>>) -> Self {
        GraphicsResourceSystem {
            video: video,
            arena: HashMap::new(),
        }
    }
}

pub fn register(resource: &mut resource::ResourceSystem,
                video: Arc<RwLock<graphics::GraphicsSystemShared>>) {
    resource.register::<Bytes>(0);
    resource.register::<Texture>(0);
    resource.register_extern_system(GraphicsResourceSystem::<graphics::TextureHandle>::new(video));
}