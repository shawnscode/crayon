use std::path::Path;
use std::sync::{Arc, RwLock};
use std::marker::PhantomData;

use crayon::{graphics, resource, application};
use super::atlas::*;
use super::errors::*;

impl_handle!(AtlasHandle);
impl_handle!(AtlasFrameHandle);

pub struct AtlasSystem {
    video: graphics::RAIIGuard,
    states: resource::Registery<Arc<RwLock<AtlasState>>>,
    resource: Arc<resource::ResourceSystemShared>,
}

impl AtlasSystem {
    pub fn new(ctx: &application::Context) -> Result<Self> {
        let video = ctx.shared::<graphics::GraphicsSystem>();
        let resource = ctx.shared::<resource::ResourceSystem>();

        Ok(AtlasSystem {
               video: graphics::RAIIGuard::new(video.clone()),
               states: resource::Registery::new(),
               resource: resource.clone(),
           })
    }

    pub fn create_from<P>(&mut self, location: resource::Location) -> Result<AtlasHandle>
        where P: AtlasParser + Send + Sync + 'static
    {
        if let Some(handle) = self.states.lookup(location) {
            self.states.inc_rc(handle);
            return Ok(handle.into());
        }

        let state = Arc::new(RwLock::new(AtlasState::NotReady));

        let loader = AtlasLoader::<P> {
            state: state.clone(),
            _phantom: PhantomData,
        };

        self.resource.load_async(loader, location.uri());

        Ok(self.states.create(location, state).into())
    }

    pub fn delete(&mut self, handle: AtlasHandle) {
        self.states.dec_rc(handle.into());
    }
}

enum AtlasState {
    NotReady,
    Ready((Atlas, Option<graphics::TextureHandle>)),
    Err(String),
}

struct AtlasLoader<P>
    where P: AtlasParser
{
    state: Arc<RwLock<AtlasState>>,
    _phantom: PhantomData<P>,
}

impl<P> resource::ResourceAsyncLoader for AtlasLoader<P>
    where P: AtlasParser + Send + Sync + 'static
{
    fn on_finished(&mut self, path: &Path, result: resource::errors::Result<&[u8]>) {
        let state = match result {
            Ok(bytes) => {
                match P::parse(bytes) {
                    Ok(atlas) => AtlasState::Ready((atlas, None)),

                    Err(error) => {
                        let err = format!("Failed to load atlas from {:?}.\n{:?}.", path, error);
                        AtlasState::Err(err)
                    }
                }
            }
            Err(error) => {
                let err = format!("Failed to load atlas from {:?}.\n{:?}.", path, error);
                AtlasState::Err(err)
            }
        };

        *self.state.write().unwrap() = state;
    }
}