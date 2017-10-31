//! Build-in assets including `Texture`, `Atlas`, `Shader` etc..

mod bytes;
mod texture;

pub use self::bytes::Bytes;
pub use self::texture::Texture;

use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use resource;
use graphics;
use utils::HashValue;

pub struct GraphicsResourceSystem<T> {
    video: Arc<graphics::GraphicsSystemShared>,
    arena: HashMap<HashValue<Path>, Arc<T>>,
}

impl<T> GraphicsResourceSystem<T> {
    pub fn new(video: Arc<graphics::GraphicsSystemShared>) -> Self {
        GraphicsResourceSystem {
            video: video,
            arena: HashMap::new(),
        }
    }
}

pub fn register(resource: &mut resource::ResourceSystem,
                video: Arc<graphics::GraphicsSystemShared>) {
    resource.register::<Bytes>(0);
    resource.register::<Texture>(0);
    resource.register_extern_system(GraphicsResourceSystem::<graphics::TextureHandle>::new(video));
}