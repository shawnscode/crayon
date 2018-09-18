use std::sync::{Arc, RwLock};
use std::thread::Builder;

use cpal::{self, EventLoop, StreamData, UnknownTypeOutputBuffer};
use crayon::math::Vector3;
use crayon::utils::HandlePool;

use assets::AudioClip;
use source::{AudioSource, AudioSourceHandle, AudioSourceSpatial, AudioSourceWrap};
use {AudioClipRegistry, Result};

pub fn mixer(clips: Arc<AudioClipRegistry>) -> Result<MixerController> {
    let device = cpal::default_output_device()
        .ok_or_else(|| format_err!("No avaiable audio output device"))?;
    let format = device
        .default_output_format()
        .expect("The device doesn't support any format.");

    let events = EventLoop::new();
    let stream = events.build_output_stream(&device, &format).unwrap();
    info!("Created audio mixer. [{:?}] {:?}.", device.name(), format);

    let cmds = Arc::new(RwLock::new(Vec::new()));
    let mut mixer = Mixer {
        channels: format.channels as u8,
        channels_iter: 0,
        sample_rate: format.sample_rate.0 as u32,
        listener: Vector3::new(0.0, 0.0, 0.0),
        sources: Vec::new(),
        rx: cmds.clone(),
        bufs: Vec::new(),
    };

    Builder::new()
        .name("Audio".into())
        .spawn(move || {
            events.run(move |id, buffer| {
                if stream != id {
                    return;
                }

                mixer.run(buffer);
            })
        }).expect("Failed to create thread for `AudioSystem`.");

    Ok(MixerController {
        clips: clips,
        sources: RwLock::new(HandlePool::new()),
        tx: cmds,
    })
}

pub fn headless(clips: Arc<AudioClipRegistry>) -> Result<MixerController> {
    let cmds = Arc::new(RwLock::new(Vec::new()));
    Ok(MixerController {
        clips: clips,
        sources: RwLock::new(HandlePool::new()),
        tx: cmds,
    })
}

pub struct MixerController {
    clips: Arc<AudioClipRegistry>,
    sources: RwLock<HandlePool<AudioSourceHandle>>,
    tx: Arc<RwLock<Vec<Command>>>,
}

impl MixerController {
    #[inline]
    pub fn create_source(&self, params: AudioSource) -> Result<AudioSourceHandle> {
        if let Some(clip) = self
            .clips
            .wait_until(params.clip)
            .ok()
            .and_then(|_| self.clips.get(params.clip, |v| v.clone()))
        {
            let handle = self.sources.write().unwrap().create();
            self.tx
                .write()
                .unwrap()
                .push(Command::CreateSource(handle, params, clip));
            Ok(handle)
        } else {
            bail!("The AudioClip {:?} is not available.", params.clip);
        }
    }

    #[inline]
    pub fn set_listener(&self, position: Vector3<f32>) {
        self.tx
            .write()
            .unwrap()
            .push(Command::UpdateListener(position));
    }

    #[inline]
    pub fn delete_source(&self, handle: AudioSourceHandle) {
        self.tx.write().unwrap().push(Command::DeleteSource(handle));
    }

    #[inline]
    pub fn update_source_volume(&self, handle: AudioSourceHandle, volume: f32) {
        self.tx
            .write()
            .unwrap()
            .push(Command::UpdateSourceVolume(handle, volume));
    }

    #[inline]
    pub fn update_source_pitch(&self, handle: AudioSourceHandle, pitch: f32) {
        self.tx
            .write()
            .unwrap()
            .push(Command::UpdateSourcePitch(handle, pitch));
    }

    #[inline]
    pub fn update_source_position(&self, handle: AudioSourceHandle, position: Vector3<f32>) {
        self.tx
            .write()
            .unwrap()
            .push(Command::UpdateSourcePosition(handle, position));
    }
}

#[derive(Debug, Clone)]
enum Command {
    UpdateListener(Vector3<f32>),
    CreateSource(AudioSourceHandle, AudioSource, Arc<AudioClip>),
    DeleteSource(AudioSourceHandle),
    UpdateSourceVolume(AudioSourceHandle, f32),
    UpdateSourcePitch(AudioSourceHandle, f32),
    UpdateSourcePosition(AudioSourceHandle, Vector3<f32>),
}

struct Mixer {
    channels: u8,
    sample_rate: u32,
    listener: Vector3<f32>,

    channels_iter: u8,
    sources: Vec<Option<AudioSourceInstance>>,
    rx: Arc<RwLock<Vec<Command>>>,
    bufs: Vec<Command>,
}

impl Mixer {
    fn run(&mut self, data: StreamData) {
        self.update();

        if let StreamData::Output { buffer } = data {
            match buffer {
                UnknownTypeOutputBuffer::U16(mut buffer) => for v in buffer.iter_mut() {
                    *v = sample_f32_to_u16(self.sample())
                },
                UnknownTypeOutputBuffer::I16(mut buffer) => for v in buffer.iter_mut() {
                    *v = sample_f32_to_i16(self.sample())
                },
                UnknownTypeOutputBuffer::F32(mut buffer) => for v in buffer.iter_mut() {
                    *v = self.sample();
                },
            }
        }
    }

    fn sample(&mut self) -> f32 {
        let mut sum = 0.0;
        for v in &mut self.sources {
            if let Some(ref source) = v {
                sum += source.sample(self.channels_iter, self.listener);
            }
        }

        self.channels_iter = (self.channels_iter + 1) % self.channels;

        if self.channels_iter == 0 {
            let sample_rate = self.sample_rate;
            for v in &mut self.sources {
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

    fn update(&mut self) {
        {
            let mut rx = self.rx.write().unwrap();
            ::std::mem::swap(&mut self.bufs, &mut rx);
        }

        for cmd in self.bufs.drain(..) {
            match cmd {
                Command::UpdateListener(position) => {
                    self.listener = position;
                }
                Command::CreateSource(handle, source, clip) => {
                    if let AudioSourceWrap::Repeat(v) = source.loops {
                        if v <= 0 {
                            continue;
                        }
                    }

                    let index = handle.index() as usize;
                    if self.sources.len() <= index {
                        self.sources.resize(index + 1, None);
                    }

                    self.sources[index] = Some(AudioSourceInstance::new(clip, source));
                }
                Command::DeleteSource(handle) => {
                    let index = handle.index() as usize;
                    if self.sources.len() > index {
                        self.sources[index] = None;
                    }
                }
                Command::UpdateSourcePitch(handle, pitch) => {
                    let index = handle.index() as usize;
                    if let Some(v) = self.sources.get_mut(index).and_then(|v| v.as_mut()) {
                        v.pitch = pitch;
                    }
                }
                Command::UpdateSourceVolume(handle, volume) => {
                    let index = handle.index() as usize;
                    if let Some(v) = self.sources.get_mut(index).and_then(|v| v.as_mut()) {
                        v.volume = volume;
                    }
                }
                Command::UpdateSourcePosition(handle, emitter) => {
                    let index = handle.index() as usize;
                    if let Some(v) = self.sources.get_mut(index).and_then(|v| v.as_mut()) {
                        if let Some(ref mut v) = v.spatial {
                            v.position = emitter;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct AudioSourceInstance {
    clip: Arc<AudioClip>,
    volume: f32,
    pitch: f32,
    loops: AudioSourceWrap,
    spatial: Option<AudioSourceSpatial>,
    iter: f32,
}

impl AudioSourceInstance {
    fn new(clip: Arc<AudioClip>, source: AudioSource) -> Self {
        AudioSourceInstance {
            clip: clip,
            volume: source.volume,
            pitch: source.pitch,
            loops: source.loops,
            spatial: source.spatial,
            iter: 0.0,
        }
    }

    fn sample(&self, channels_iter: u8, listener: Vector3<f32>) -> f32 {
        let mut idx = (self.iter as usize) * (self.clip.channels as usize);
        idx += (channels_iter % self.clip.channels) as usize;

        if idx < self.clip.pcm.len() {
            let mut v = sample_i16_to_f32(self.clip.pcm[idx]) * self.volume;

            if let Some(spatial) = self.spatial {
                v *= spatial.volume(listener);
            }

            v
        } else {
            0.0
        }
    }

    fn advance(&mut self, sample_rate: u32) -> bool {
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
fn sample_i16_to_f32(sample: i16) -> f32 {
    if sample < 0 {
        sample as f32 / -(::std::i16::MIN as f32)
    } else {
        sample as f32 / ::std::i16::MAX as f32
    }
}

#[inline]
fn sample_f32_to_i16(sample: f32) -> i16 {
    if sample >= 0.0 {
        (sample * ::std::i16::MAX as f32) as i16
    } else {
        (-sample * ::std::i16::MIN as f32) as i16
    }
}

#[inline]
fn sample_f32_to_u16(sample: f32) -> u16 {
    (((sample + 1.0) * 0.5) * ::std::u16::MAX as f32).round() as u16
}
