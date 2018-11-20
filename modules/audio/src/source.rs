use crayon::math::prelude::Vector3;

use assets::prelude::AudioClipHandle;

impl_handle!(AudioSourceHandle);

#[derive(Debug, Copy, Clone)]
pub struct AudioSource {
    /// Set the sound effect handle.
    pub clip: AudioClipHandle,
    /// Set the volume of a playing sound.
    pub volume: f32,
    /// Set the frequency-shift of a playing sound.
    pub pitch: f32,
    /// Set the wrap mode of playing sound.
    pub loops: AudioSourceWrap,
    /// Sets the spatial information of playing sound.
    pub attenuation: Option<AudioSourceAttenuation>,
}

impl From<AudioClipHandle> for AudioSource {
    fn from(clip: AudioClipHandle) -> Self {
        AudioSource {
            clip: clip,
            volume: 1.0,
            pitch: 1.0,
            loops: AudioSourceWrap::Repeat(1),
            attenuation: None,
        }
    }
}

/// The wrap mode of audio source.
#[derive(Debug, Copy, Clone)]
pub enum AudioSourceWrap {
    Repeat(u32),
    Infinite,
}

#[derive(Debug, Copy, Clone)]
pub struct AudioSourceAttenuation {
    /// Set the emiiter position of playing sound.
    pub position: Vector3<f32>,
    /// The minimum distance is the distance under which the sound will be
    /// heard at its maximum volume.
    pub minimum_distance: f32,
    /// The attenuation is a multiplicative factor. The greater the attenuation,
    /// the less it will be heard when the sound moves away from the listener.
    ///
    /// To get a non-attenuated sound, you can use 0.
    pub attenuation: f32,
}

impl AudioSourceAttenuation {
    pub fn new(minimum_distance: f32, attenuation: f32) -> Self {
        assert!(minimum_distance > 0.0 && attenuation >= 0.0);

        AudioSourceAttenuation {
            position: Vector3::new(0.0, 0.0, 0.0),
            minimum_distance: minimum_distance,
            attenuation: attenuation,
        }
    }

    pub fn volume<T>(&self, listener: T) -> f32
    where
        T: Into<Vector3<f32>>,
    {
        use crayon::math::prelude::InnerSpace;

        let distance = (listener.into() - self.position)
            .magnitude()
            .max(self.minimum_distance);

        let attenuation = self.attenuation * (distance - self.minimum_distance);
        self.minimum_distance / (self.minimum_distance + attenuation)
    }
}
