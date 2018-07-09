use std;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::{Arc, RwLock};

use math;
use resource::prelude::*;
use resource::utils::registery::Registery;
use video::assets::texture::*;
use video::assets::{AssetState, AssetTextureState};
use video::backend::frame::{DoubleFrame, PreFrameTask};

/// Parsed texture from `TextureParser`.
pub struct TextureData {
    pub format: TextureFormat,
    pub dimensions: math::Vector2<u32>,
    pub data: Vec<u8>,
}

/// Parse bytes into texture.
pub trait TextureParser: Send + 'static {
    type Error: std::error::Error + std::fmt::Debug;

    fn parse(bytes: &[u8]) -> std::result::Result<TextureData, Self::Error>;
}

#[doc(hidden)]
pub(crate) struct TextureLoader<T>
where
    T: TextureParser,
{
    handle: TextureHandle,
    params: TextureParams,
    textures: Arc<RwLock<Registery<AssetTextureState>>>,
    frames: Arc<DoubleFrame>,
    _phantom: PhantomData<T>,
}

impl<T> TextureLoader<T>
where
    T: TextureParser,
{
    pub fn new(
        handle: TextureHandle,
        params: TextureParams,
        textures: Arc<RwLock<Registery<AssetTextureState>>>,
        frames: Arc<DoubleFrame>,
    ) -> Self {
        TextureLoader {
            handle: handle,
            params: params,
            textures: textures,
            frames: frames,
            _phantom: PhantomData,
        }
    }
}

impl<T> ResourceTask for TextureLoader<T>
where
    T: TextureParser,
{
    fn execute(mut self, driver: &mut ResourceFS, path: &Path) {
        let state = match driver.load(path) {
            Ok(bytes) => match T::parse(bytes) {
                Ok(texture) => {
                    self.params.dimensions = texture.dimensions;
                    self.params.format = texture.format;

                    let mut frame = self.frames.front();
                    let ptr = frame.buf.extend_from_slice(&texture.data);
                    let task = PreFrameTask::CreateTexture(self.handle, self.params, Some(ptr));
                    frame.pre.push(task);

                    AssetState::ready(self.params)
                }
                Err(error) => {
                    let error = format!("Failed to load texture at {:?}.\n{:?}", path, error);
                    AssetState::Err(error)
                }
            },
            Err(error) => {
                let error = format!("Failed to load texture at {:?}.\n{:?}", path, error);
                AssetState::Err(error)
            }
        };

        {
            let mut textures = self.textures.write().unwrap();
            if let Some(texture) = textures.get_mut(*self.handle) {
                *texture = state;
            }
        }
    }
}
