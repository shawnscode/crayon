use std::sync::{Arc, RwLock};

use crayon::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crayon::errors::Result;
use crayon::math::prelude::Vector3;
use crayon::res::utils::prelude::{ResourcePool, ResourceState};
use crayon::uuid::Uuid;

use super::assets::prelude::{AudioClipHandle, AudioClipLoader};
use super::mixer::Mixer;
use super::source::{AudioSource, AudioSourceHandle};

/// The centralized management of audio sub-system.
pub struct AudioSystem {
    lis: LifecycleListenerHandle,
    clips: Arc<RwLock<ResourcePool<AudioClipHandle, AudioClipLoader>>>,
    mixer: Mixer,
}

struct AudioState {
    clips: Arc<RwLock<ResourcePool<AudioClipHandle, AudioClipLoader>>>,
}

impl LifecycleListener for AudioState {
    fn on_pre_update(&mut self) -> Result<()> {
        self.clips.write().unwrap().advance()?;
        Ok(())
    }
}

impl Drop for AudioSystem {
    fn drop(&mut self) {
        crayon::application::detach(self.lis);
    }
}

impl AudioSystem {
    pub fn new() -> Result<Self> {
        let clips = Arc::new(RwLock::new(ResourcePool::new(AudioClipLoader::new())));
        let mixer = if crayon::application::headless() {
            Mixer::headless(clips.clone())?
        } else {
            Mixer::new(clips.clone())?
        };

        let state = AudioState {
            clips: clips.clone(),
        };

        Ok(AudioSystem {
            lis: crayon::application::attach(state),
            clips: clips,
            mixer: mixer,
        })
    }

    /// Sets the position of listener.
    #[inline]
    pub fn set_listener<T>(&self, position: T)
    where
        T: Into<Vector3<f32>>,
    {
        self.mixer.set_listener(position.into());
    }

    /// Creates a clip object from file asynchronously.
    #[inline]
    pub fn create_clip_from<T: AsRef<str>>(&self, url: T) -> Result<AudioClipHandle> {
        self.clips.write().unwrap().create_from(url)
    }

    /// Creates a clip object from file asynchronously.
    #[inline]
    pub fn create_clip_from_uuid(&self, uuid: Uuid) -> Result<AudioClipHandle> {
        self.clips.write().unwrap().create_from_uuid(uuid)
    }

    #[inline]
    pub fn clip_state(&self, handle: AudioClipHandle) -> ResourceState {
        self.clips.read().unwrap().state(handle)
    }

    /// Deletes a `AudioClip` resource from `AudioSystem`.
    #[inline]
    pub fn delete_clip(&self, handle: AudioClipHandle) {
        self.clips.write().unwrap().delete(handle);
    }

    /// Plays a audio source, returning a `AudioSourceHandle` for it.
    #[inline]
    pub fn play<T>(&self, params: T) -> Result<AudioSourceHandle>
    where
        T: Into<AudioSource>,
    {
        self.mixer.create_source(params.into())
    }

    /// Stops a played audio source.
    #[inline]
    pub fn stop(&self, handle: AudioSourceHandle) {
        self.mixer.delete_source(handle);
    }

    /// Sets the emiiter position of playing sound.
    #[inline]
    pub fn set_position<T>(&self, handle: AudioSourceHandle, position: T)
    where
        T: Into<Vector3<f32>>,
    {
        self.mixer.set_position(handle, position.into());
    }

    /// Sets the volume of a playing sound.
    #[inline]
    pub fn set_volume(&self, handle: AudioSourceHandle, volume: f32) {
        self.mixer.set_volume(handle, volume);
    }

    /// Sets the frequency-shift of a playing sound.
    #[inline]
    pub fn set_pitch(&self, handle: AudioSourceHandle, pitch: f32) {
        self.mixer.set_pitch(handle, pitch);
    }
}
