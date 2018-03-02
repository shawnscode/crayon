use std;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;

use resource;
use resource::utils::registery::Registery;
use graphics::assets::texture::*;
use graphics::assets::{AssetState, AssetTextureState};
use graphics::backend::frame::{DoubleFrame, PreFrameTask};

/// Parsed texture from `TextureParser`.
pub struct TextureData {
    pub format: TextureFormat,
    pub dimensions: (u16, u16),
    pub data: Vec<u8>,
}

/// Parse bytes into texture.
pub trait TextureParser {
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

impl<T> resource::ResourceAsyncLoader for TextureLoader<T>
where
    T: TextureParser + Send + Sync + 'static,
{
    fn on_finished(mut self, path: &Path, result: resource::errors::Result<&[u8]>) {
        let state = match result {
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
