use std::sync::Arc;

use crayon::math::prelude::Vector3;

use assets::prelude::AudioClip;
use source::{AudioSource, AudioSourceAttenuation, AudioSourceHandle, AudioSourceWrap};

use super::Command;

pub struct Sampler {
    channels: u8,
    sample_rate: u32,
    listener: Vector3<f32>,
    channels_iter: u8,
    samplers: Vec<Option<AudioSourceSampler>>,
}

impl Sampler {
    pub fn new(channels: u8, sample_rate: u32) -> Self {
        Sampler {
            channels: channels,
            sample_rate: sample_rate,
            listener: Vector3::new(0.0, 0.0, 0.0),
            channels_iter: 0,
            samplers: Vec::new(),
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn sample_u16(&mut self) -> u16 {
        sample_f32_to_u16(self.sample())
    }

    #[allow(dead_code)]
    #[inline]
    pub fn sample_i16(&mut self) -> i16 {
        sample_f32_to_i16(self.sample())
    }

    pub fn sample(&mut self) -> f32 {
        let mut sum = 0.0;
        for v in &mut self.samplers {
            if let Some(ref source) = v {
                sum += source.sample(self.channels_iter, self.listener);
            }
        }

        self.channels_iter = (self.channels_iter + 1) % self.channels;

        if self.channels_iter == 0 {
            let sample_rate = self.sample_rate;
            for v in &mut self.samplers {
                let free = v
                    .as_mut()
                    .map(|source| source.advance(sample_rate))
                    .unwrap_or(false);

                if free {
                    *v = None;
                }
            }
        }

        sum
    }

    pub fn update<T: Iterator<Item = Command>>(&mut self, bufs: T) -> bool {
        for cmd in bufs {
            match cmd {
                Command::SetListener(position) => self.set_listener(position),
                Command::CreateSource(handle, source, c) => self.create_source(handle, source, c),
                Command::DeleteSource(handle) => self.delete_source(handle),
                Command::SetPitch(handle, pitch) => self.set_pitch(handle, pitch),
                Command::SetVolume(handle, volume) => self.set_volume(handle, volume),
                Command::SetPosition(handle, emitter) => self.set_position(handle, emitter),
                Command::Discard => {
                    return false;
                }
            }
        }

        true
    }

    pub fn create_source(
        &mut self,
        handle: AudioSourceHandle,
        source: AudioSource,
        clip: Arc<AudioClip>,
    ) {
        if let AudioSourceWrap::Repeat(v) = source.loops {
            if v <= 0 {
                return;
            }
        }

        let index = handle.index() as usize;
        if self.samplers.len() <= index {
            self.samplers.resize(index + 1, None);
        }

        self.samplers[index] = Some(AudioSourceSampler::new(clip, source));
    }

    #[inline]
    pub fn delete_source(&mut self, handle: AudioSourceHandle) {
        let index = handle.index() as usize;
        if self.samplers.len() > index {
            self.samplers[index] = None;
        }
    }

    #[inline]
    pub fn set_listener(&mut self, position: Vector3<f32>) {
        self.listener = position;
    }

    #[inline]
    pub fn set_volume(&mut self, handle: AudioSourceHandle, volume: f32) {
        let index = handle.index() as usize;
        if let Some(v) = self.samplers.get_mut(index).and_then(|v| v.as_mut()) {
            v.set_volume(volume);
        }
    }

    #[inline]
    pub fn set_pitch(&mut self, handle: AudioSourceHandle, pitch: f32) {
        let index = handle.index() as usize;
        if let Some(v) = self.samplers.get_mut(index).and_then(|v| v.as_mut()) {
            v.set_pitch(pitch);
        }
    }

    #[inline]
    pub fn set_position(&mut self, handle: AudioSourceHandle, position: Vector3<f32>) {
        let index = handle.index() as usize;
        if let Some(v) = self.samplers.get_mut(index).and_then(|v| v.as_mut()) {
            v.set_position(position);
        }
    }
}

#[derive(Clone)]
pub struct AudioSourceSampler {
    clip: Arc<AudioClip>,
    volume: f32,
    pitch: f32,
    loops: AudioSourceWrap,
    attenuation: Option<AudioSourceAttenuation>,
    iter: f32,
}

impl AudioSourceSampler {
    pub fn new(clip: Arc<AudioClip>, source: AudioSource) -> Self {
        AudioSourceSampler {
            clip: clip,
            volume: source.volume,
            pitch: source.pitch,
            loops: source.loops,
            attenuation: source.attenuation,
            iter: 0.0,
        }
    }

    #[inline]
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }

    #[inline]
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    #[inline]
    pub fn set_position(&mut self, position: Vector3<f32>) {
        if let Some(ref mut v) = self.attenuation {
            v.position = position;
        }
    }

    pub fn sample(&self, channels_iter: u8, listener: Vector3<f32>) -> f32 {
        let mut idx = (self.iter as usize) * (self.clip.channels as usize);
        idx += (channels_iter % self.clip.channels) as usize;

        if idx < self.clip.pcm.len() {
            let mut v = sample_i16_to_f32(self.clip.pcm[idx]) * self.volume;

            if let Some(attenuation) = self.attenuation {
                v *= attenuation.volume(listener);
            }

            v
        } else {
            0.0
        }
    }

    pub fn advance(&mut self, sample_rate: u32) -> bool {
        let pitch = self.pitch.min(100.0).max(0.01);
        self.iter += pitch * (self.clip.sample_rate as f32) / (sample_rate as f32);

        let samples = (self.clip.pcm.len() as f32) / (self.clip.channels as f32);
        while (self.iter as usize) * (self.clip.channels as usize) >= self.clip.pcm.len() {
            match self.loops {
                AudioSourceWrap::Repeat(ref mut c) => {
                    if *c > 1 {
                        *c -= 1;
                        self.iter -= samples;
                    } else {
                        return true;
                    }
                }
                AudioSourceWrap::Infinite => {
                    self.iter -= samples;
                }
            }
        }

        false
    }
}

#[inline]
pub fn sample_i16_to_f32(sample: i16) -> f32 {
    if sample < 0 {
        sample as f32 / -(::std::i16::MIN as f32)
    } else {
        sample as f32 / ::std::i16::MAX as f32
    }
}

#[allow(dead_code)]
#[inline]
pub fn sample_f32_to_i16(sample: f32) -> i16 {
    if sample >= 0.0 {
        (sample * ::std::i16::MAX as f32) as i16
    } else {
        (-sample * ::std::i16::MIN as f32) as i16
    }
}

#[allow(dead_code)]
#[inline]
pub fn sample_f32_to_u16(sample: f32) -> u16 {
    (((sample + 1.0) * 0.5) * ::std::u16::MAX as f32).round() as u16
}
