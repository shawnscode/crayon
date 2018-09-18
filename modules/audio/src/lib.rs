extern crate cpal;
extern crate lewton;
#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;

pub mod assets;
mod mixer;
pub mod source;

use cpal::{EventLoop, StreamId};
use std::sync::Arc;
use std::thread::Builder;

use crayon::application::Context;
use crayon::math::Vector3;
use crayon::res::prelude::{Location, ResourceSystemShared};
use crayon::res::registry::Registry;

use self::assets::{AudioClipHandle, AudioClipLoader};
use self::mixer::{Mixer, MixerController};
use self::source::{AudioSource, AudioSourceHandle};

pub mod prelude {
    pub use super::{AudioSystem, AudioSystemShared};
    pub use assets::AudioClipHandle;
    pub use source::{AudioSource, AudioSourceHandle, AudioSourceSpatial, AudioSourceWrap};
}

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub type AudioClipRegistry = Registry<AudioClipHandle, AudioClipLoader>;

pub struct AudioSystem {
    shared: Arc<AudioSystemShared>,
}

impl AudioSystem {
    /// Setups the audio system with default audio output device.
    pub fn new(ctx: &Context) -> Result<Self> {
        let (mixer, shared) = AudioSystemShared::new(ctx.res.clone())?;
        let shared = Arc::new(shared);

        Self::run(mixer, shared.clone());
        Ok(AudioSystem { shared: shared })
    }

    pub fn shared(&self) -> Arc<AudioSystemShared> {
        self.shared.clone()
    }

    fn run(mut mixer: Mixer, audio: Arc<AudioSystemShared>) {
        Builder::new()
            .name("Audio".into())
            .spawn(move || {
                audio.events.run(|stream, buffer| {
                    if audio.stream != stream {
                        return;
                    }

                    mixer.run(buffer);
                })
            }).expect("Failed to create thread for `AudioSystem`.");
    }
}

pub struct AudioSystemShared {
    clips: Arc<AudioClipRegistry>,
    mixer: MixerController,
    events: EventLoop,
    stream: StreamId,
}

impl AudioSystemShared {
    fn new(res: Arc<ResourceSystemShared>) -> Result<(Mixer, Self)> {
        let events = EventLoop::new();

        let device = cpal::default_output_device()
            .ok_or_else(|| format_err!("No avaiable audio output device"))?;

        let format = device
            .default_output_format()
            .expect("The device doesn't support any format.");

        let stream = events.build_output_stream(&device, &format).unwrap();
        info!("Create audio system on {:?} ({:?}).", device.name(), format);

        let clips = Arc::new(AudioClipRegistry::new(res, AudioClipLoader::new()));
        let (mixer, mixer_controller) = mixer::mixer(
            format.channels as u8,
            format.sample_rate.0 as u32,
            clips.clone(),
        );

        Ok((
            mixer,
            AudioSystemShared {
                clips: clips,
                mixer: mixer_controller,
                events: events,
                stream: stream,
            },
        ))
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
