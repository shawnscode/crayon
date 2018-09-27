extern crate cpal;
extern crate lewton;
#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;

pub mod assets;
mod mixer;
pub mod source;

use std::sync::Arc;

use crayon::math::prelude::Vector3;
use crayon::res::prelude::*;

use self::assets::{AudioClipHandle, AudioClipLoader};
use self::mixer::MixerController;
use self::source::{AudioSource, AudioSourceHandle};

pub mod prelude {
    pub use super::{AudioSystem, AudioSystemShared};
    pub use assets::AudioClipHandle;
    pub use source::{AudioSource, AudioSourceHandle, AudioSourceSpatial, AudioSourceWrap};
}

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
pub type AudioClipRegistry = Registry<AudioClipHandle, AudioClipLoader>;

/// The centralized management of audio sub-system.
pub struct AudioSystem {
    shared: Arc<AudioSystemShared>,
}

impl AudioSystem {
    /// Setups the audio system with default audio output device.
    pub fn new(res: Arc<ResourceSystemShared>) -> Result<Self> {
        let shared = Arc::new(AudioSystemShared::new(res)?);
        Ok(AudioSystem { shared: shared })
    }

    pub fn headless<T>(res: T) -> Result<Self>
    where
        T: Into<Option<Arc<ResourceSystemShared>>>,
    {
        let shared = Arc::new(AudioSystemShared::headless(res)?);
        Ok(AudioSystem { shared: shared })
    }

    pub fn shared(&self) -> Arc<AudioSystemShared> {
        self.shared.clone()
    }
}

/// The multi-thread friendly parts of `AudioSystem`.
pub struct AudioSystemShared {
    clips: Arc<AudioClipRegistry>,
    mixer: MixerController,
}

impl AudioSystemShared {
    fn new(res: Arc<ResourceSystemShared>) -> Result<Self> {
        let clips = Arc::new(AudioClipRegistry::new(res, AudioClipLoader::new()));
        let mixer_controller = mixer::mixer(clips.clone())?;

        Ok(AudioSystemShared {
            clips: clips,
            mixer: mixer_controller,
        })
    }

    fn headless<T>(res: T) -> Result<Self>
    where
        T: Into<Option<Arc<ResourceSystemShared>>>,
    {
        let res = res.into().unwrap_or_else(|| {
            use crayon::{res, sched};

            let sched = sched::ScheduleSystem::new(1, None, None);
            res::ResourceSystem::new(sched.shared()).unwrap().shared()
        });

        let clips = Arc::new(AudioClipRegistry::new(res, AudioClipLoader::new()));
        let mixer_controller = mixer::headless(clips.clone())?;
        Ok(AudioSystemShared {
            clips: clips,
            mixer: mixer_controller,
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

    /// Creates a `AudioClip` resource from specified location.
    #[inline]
    pub fn create_clip_from<'a, T>(&'a self, location: T) -> Result<AudioClipHandle>
    where
        T: Into<Location<'a>>,
    {
        self.clips.create_from(location.into())
    }

    /// Deletes a `AudioClip` resource from `AudioSystem`.
    #[inline]
    pub fn delete_clip(&self, handle: AudioClipHandle) {
        self.clips.delete(handle);
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
        self.mixer.update_source_position(handle, position.into());
    }

    /// Sets the volume of a playing sound.
    #[inline]
    pub fn set_volume(&self, handle: AudioSourceHandle, volume: f32) {
        self.mixer.update_source_volume(handle, volume);
    }

    /// Sets the frequency-shift of a playing sound.
    #[inline]
    pub fn set_pitch(&self, handle: AudioSourceHandle, pitch: f32) {
        self.mixer.update_source_pitch(handle, pitch);
    }
}
