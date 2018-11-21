#[cfg(not(target_arch = "wasm32"))]
mod cpal;
#[cfg(target_arch = "wasm32")]
mod webaudio;

mod headless;
mod sampler;

use std::sync::{Arc, RwLock};

use crayon::errors::Result;
use crayon::math::prelude::Vector3;
use crayon::res::utils::prelude::ResourcePool;
use crayon::utils::prelude::HandlePool;

use assets::prelude::{AudioClip, AudioClipHandle, AudioClipLoader};
use source::{AudioSource, AudioSourceHandle};

pub struct Mixer {
    sources: RwLock<HandlePool<AudioSourceHandle>>,
    tx: Arc<RwLock<Vec<Command>>>,
    clips: Arc<RwLock<ResourcePool<AudioClipHandle, AudioClipLoader>>>,
}

impl Mixer {
    pub fn new(clips: Arc<RwLock<ResourcePool<AudioClipHandle, AudioClipLoader>>>) -> Result<Self> {
        let tx = Arc::new(RwLock::new(Vec::new()));

        #[cfg(not(target_arch = "wasm32"))]
        cpal::run(tx.clone())?;

        #[cfg(target_arch = "wasm32")]
        webaudio::run(tx.clone())?;

        Ok(Mixer {
            sources: RwLock::new(HandlePool::new()),
            tx: tx,
            clips: clips,
        })
    }

    pub fn headless(
        clips: Arc<RwLock<ResourcePool<AudioClipHandle, AudioClipLoader>>>,
    ) -> Result<Self> {
        let tx = Arc::new(RwLock::new(Vec::new()));
        headless::run(tx.clone())?;

        Ok(Mixer {
            sources: RwLock::new(HandlePool::new()),
            tx: tx,
            clips: clips,
        })
    }
}

impl Drop for Mixer {
    fn drop(&mut self) {
        let cmd = Command::Discard;
        self.tx.write().unwrap().push(cmd);
    }
}

impl Mixer {
    #[inline]
    pub fn create_source(&self, params: AudioSource) -> Result<AudioSourceHandle> {
        if let Some(clip) = self.clips.read().unwrap().resource(params.clip).cloned() {
            let handle = self.sources.write().unwrap().create();
            let cmd = Command::CreateSource(handle, params, clip);
            self.tx.write().unwrap().push(cmd);
            Ok(handle)
        } else {
            bail!("The AudioClip {:?} is not available.", params.clip);
        }
    }

    #[inline]
    pub fn set_listener(&self, position: Vector3<f32>) {
        let cmd = Command::SetListener(position);
        self.tx.write().unwrap().push(cmd);
    }

    #[inline]
    pub fn delete_source(&self, handle: AudioSourceHandle) {
        let cmd = Command::DeleteSource(handle);
        self.tx.write().unwrap().push(cmd);
    }

    #[inline]
    pub fn set_volume(&self, handle: AudioSourceHandle, volume: f32) {
        let cmd = Command::SetVolume(handle, volume);
        self.tx.write().unwrap().push(cmd);
    }

    #[inline]
    pub fn set_pitch(&self, handle: AudioSourceHandle, pitch: f32) {
        let cmd = Command::SetPitch(handle, pitch);
        self.tx.write().unwrap().push(cmd);
    }

    #[inline]
    pub fn set_position(&self, handle: AudioSourceHandle, position: Vector3<f32>) {
        let cmd = Command::SetPosition(handle, position);
        self.tx.write().unwrap().push(cmd);
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    SetListener(Vector3<f32>),
    CreateSource(AudioSourceHandle, AudioSource, Arc<AudioClip>),
    DeleteSource(AudioSourceHandle),
    SetVolume(AudioSourceHandle, f32),
    SetPitch(AudioSourceHandle, f32),
    SetPosition(AudioSourceHandle, Vector3<f32>),
    Discard,
}
