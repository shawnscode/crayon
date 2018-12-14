#[cfg(not(target_arch = "wasm32"))]
extern crate cpal;

#[cfg(target_arch = "wasm32")]
extern crate js_sys;
#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;
#[cfg(target_arch = "wasm32")]
extern crate web_sys;

#[macro_use]
extern crate crayon;
#[macro_use]
extern crate failure;
extern crate lewton;

pub mod assets;
pub mod source;

mod mixer;
mod system;

pub mod prelude {
    pub use assets::prelude::AudioClipHandle;
    pub use source::{AudioSource, AudioSourceAttenuation, AudioSourceHandle, AudioSourceWrap};
}

pub use self::inside::{discard, setup};

use crayon::errors::Result;
use crayon::math::prelude::Vector3;
use crayon::res::prelude::ResourceState;
use crayon::uuid::Uuid;

use self::assets::prelude::AudioClipHandle;
use self::inside::ctx;
use self::source::{AudioSource, AudioSourceHandle};

/// Sets the position of listener.
#[inline]
pub fn set_listener<T>(position: T)
where
    T: Into<Vector3<f32>>,
{
    ctx().set_listener(position);
}

/// Creates a clip object from file asynchronously.
#[inline]
pub fn create_clip_from<T: AsRef<str>>(url: T) -> Result<AudioClipHandle> {
    ctx().create_clip_from(url)
}

/// Creates a clip object from file asynchronously.
#[inline]
pub fn create_clip_from_uuid(uuid: Uuid) -> Result<AudioClipHandle> {
    ctx().create_clip_from_uuid(uuid)
}

#[inline]
pub fn clip_state(handle: AudioClipHandle) -> ResourceState {
    ctx().clip_state(handle)
}

/// Deletes a `AudioClip` resource from `AudioSystem`.
#[inline]
pub fn delete_clip(handle: AudioClipHandle) {
    ctx().delete_clip(handle);
}

/// Plays a audio source, returning a `AudioSourceHandle` for it.
#[inline]
pub fn play<T>(params: T) -> Result<AudioSourceHandle>
where
    T: Into<AudioSource>,
{
    ctx().play(params)
}

/// Stops a played audio source.
#[inline]
pub fn stop(handle: AudioSourceHandle) {
    ctx().stop(handle)
}

/// Sets the emiiter position of playing sound.
#[inline]
pub fn set_position<T>(handle: AudioSourceHandle, position: T)
where
    T: Into<Vector3<f32>>,
{
    ctx().set_position(handle, position)
}

/// Sets the volume of a playing sound.
#[inline]
pub fn set_volume(handle: AudioSourceHandle, volume: f32) {
    ctx().set_volume(handle, volume);
}

/// Sets the frequency-shift of a playing sound.
#[inline]
pub fn set_pitch(handle: AudioSourceHandle, pitch: f32) {
    ctx().set_pitch(handle, pitch);
}

mod inside {
    use super::system::AudioSystem;

    static mut CTX: *const AudioSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static AudioSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "audio system has not been initialized properly."
            );

            &*CTX
        }
    }

    /// Setup the world system.
    pub fn setup() -> Result<(), failure::Error> {
        unsafe {
            debug_assert!(CTX.is_null(), "duplicated setup of audio system.");

            let ctx = AudioSystem::new()?;
            CTX = Box::into_raw(Box::new(ctx));
            Ok(())
        }
    }

    /// Discard the world system.
    pub fn discard() {
        unsafe {
            if CTX.is_null() {
                return;
            }

            drop(Box::from_raw(CTX as *mut AudioSystem));
            CTX = std::ptr::null();
        }
    }
}
