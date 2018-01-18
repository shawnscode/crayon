use std;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;

use resource;
use graphics::assets::texture::*;
use graphics::backend::frame::{DoubleFrame, PreFrameTask};

/// Parsed texture from `TextureParser`.
pub struct TextureData {
    pub format: TextureFormat,
    pub dimensions: (u32, u32),
    pub data: Vec<u8>,
}

/// Parse bytes into texture.
pub trait TextureParser {
    type Error: std::error::Error + std::fmt::Debug;

    fn parse(bytes: &[u8]) -> std::result::Result<TextureData, Self::Error>;
}

#[doc(hidden)]
#[derive(PartialEq, Eq)]
pub(crate) enum TextureState {
    NotReady,
    Ready,
    Err(String),
}

#[doc(hidden)]
pub(crate) struct TextureLoader<T>
    where T: TextureParser
{
    handle: TextureHandle,
    setup: TextureSetup,
    state: Arc<RwLock<TextureState>>,
    frames: Arc<DoubleFrame>,
    _phantom: PhantomData<T>,
}

impl<T> TextureLoader<T>
    where T: TextureParser
{
    pub fn new(handle: TextureHandle,
               state: Arc<RwLock<TextureState>>,
               setup: TextureSetup,
               frames: Arc<DoubleFrame>)
               -> Self {
        TextureLoader {
            handle: handle,
            setup: setup,
            state: state,
            frames: frames,
            _phantom: PhantomData,
        }
    }
}

impl<T> resource::ResourceAsyncLoader for TextureLoader<T>
    where T: TextureParser + Send + Sync + 'static
{
    fn on_finished(mut self, path: &Path, result: resource::errors::Result<&[u8]>) {
        let state = match result {
            Ok(bytes) => {
                match T::parse(bytes) {
                    Ok(texture) => {
                        self.setup.dimensions = texture.dimensions;
                        self.setup.format = texture.format;

                        let mut frame = self.frames.front();
                        let ptr = frame.buf.extend_from_slice(&texture.data);
                        let task = PreFrameTask::CreateTexture(self.handle, self.setup, Some(ptr));
                        frame.pre.push(task);

                        TextureState::Ready
                    }
                    Err(error) => {
                        let error = format!("Failed to load texture at {:?}.\n{:?}", path, error);
                        TextureState::Err(error)
                    }
                }
            }
            Err(error) => {
                let error = format!("Failed to load texture at {:?}.\n{:?}", path, error);
                TextureState::Err(error)
            }
        };

        *self.state.write().unwrap() = state;
    }
}
